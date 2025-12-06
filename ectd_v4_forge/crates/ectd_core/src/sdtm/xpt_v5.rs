use std::io::{self, Write};
use chrono::{Datelike, Utc};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum XptVarType {
    Numeric,
    Character,
}

/// Converts IEEE 754 f64 to IBM 370 floating point format (8 bytes).
///
/// Format:
/// Byte 0: Sign bit (1) + Exponent (7)
/// Bytes 1-7: Fraction (56 bits)
///
/// Exponent is base 16, bias 64.
/// Value = (-1)^S * 16^(E-64) * 0.F
fn to_ibm_370(val: f64) -> [u8; 8] {
    if val == 0.0 || val == -0.0 {
        return [0; 8];
    }

    let bits = val.to_bits();
    let sign = (bits >> 63) as u8;
    let ieee_exp = ((bits >> 52) & 0x7FF) as i32;
    let ieee_mant = bits & 0xFFFFFFFFFFFFF;

    // Handle denormals: treat as 0 for this implementation
    if ieee_exp == 0 {
        return [0; 8];
    }

    // Add implicit leading 1
    // IEEE mantissa is 1.xxxxx
    // We treat it as integer: 1xxxxx... (53 bits)
    let mut mantissa = ieee_mant | (1u64 << 52);

    // Calculate pure base-2 exponent
    // value = mantissa * 2^(ieee_exp - 1023 - 52)
    let mut exp2 = ieee_exp - 1023 - 52;

    // We want to represent value as: Fraction * 16^(ibm_exp - 64)
    // Fraction is 0.F (interpreted as mantissa * 2^-56)
    // So value = mantissa * 2^-56 * 16^(ibm_exp - 64)
    // value = mantissa * 2^-56 * 2^(4*ibm_exp - 256)
    // value = mantissa * 2^(4*ibm_exp - 312)

    // We have value = mantissa * 2^exp2
    // So exp2 = 4*ibm_exp - 312 + adjustment
    // We need to shift mantissa to align.

    // Let's use the property: 16^E = 2^(4E).
    // We want exp2 to be a multiple of 4?
    // Actually, let's find the target IBM exponent first.
    // value = m_ieee * 2^(e_ieee)
    // value approx 2^(e_ieee).
    // ibm_exp approx (e_ieee) / 4.

    // Let's align exp2 to be divisible by 4 using Euclidean remainder.
    // We want exp2_aligned such that exp2_aligned % 4 == 0
    // And exp2_aligned <= exp2 (so we shift mantissa left, increasing it).
    // shift = exp2 - exp2_aligned.

    // Using rem_euclid:
    // r = exp2.rem_euclid(4). This is 0, 1, 2, 3.
    // If r=1, exp2 = 4k + 1. We want 4k. shift = 1.
    // mantissa <<= 1. exp2 -= 1.

    let shift = exp2.rem_euclid(4);
    mantissa <<= shift;
    exp2 -= shift;

    // Now exp2 is divisible by 4.
    // value = mantissa * 2^exp2
    // value = mantissa * 16^(exp2 / 4)
    
    // We want form: (mantissa_final * 2^-56) * 16^(ibm_exp - 64)
    // value = mantissa * 16^(exp2 / 4)
    //       = (mantissa * 2^56 / 2^56) * 16^(exp2 / 4)
    //       = (mantissa / 2^56) * 2^56 * 16^(exp2 / 4)
    //       = (mantissa / 2^56) * 16^14 * 16^(exp2 / 4)
    //       = (mantissa / 2^56) * 16^(exp2/4 + 14)
    
    // So ibm_exp - 64 = exp2/4 + 14
    // ibm_exp = exp2/4 + 78
    
    let mut ibm_exp = (exp2 / 4) + 78;

    // Check bounds
    if ibm_exp > 127 {
        ibm_exp = 127;
        mantissa = 0xFFFFFFFFFFFFFF;
    } else if ibm_exp < 0 {
        return [0; 8];
    }

    // Pack into bytes
    let mut out = [0u8; 8];
    out[0] = (sign << 7) | (ibm_exp as u8);

    // Mantissa is currently in lower bits of u64.
    // We want it in bytes 1-7 (56 bits).
    // But verify: is mantissa < 2^56?
    // Original 53 bits. Max shift 3. 53+3 = 56.
    // So mantissa fits exactly in 56 bits.
    // 1xxxxxxxx... (56 bits max)
    // We write it as big endian.
    
    let m_bytes = mantissa.to_be_bytes(); 
    // m_bytes is 8 bytes [0, 1, 2, 3, 4, 5, 6, 7]
    // Since mantissa fits in 56 bits, the top byte (0) should be 0.
    // The data is in bytes 1..7.
    // Wait, let's verify.
    // If mantissa is 1 (bit 0 set), to_be_bytes ends with 1.
    // If mantissa has bit 55 set (top of 56 bits),
    // 1 << 55.
    // Byte 0 is (1<<55) >> 56 = 0.
    // Byte 1 is (1<<55) >> 48 = (1<<7) = 128.
    // So the data IS in bytes 1-7 of the 8-byte array?
    // Let's check:
    // u64: [b0, b1, b2, b3, b4, b5, b6, b7]
    // Value = b0*2^56 + ...
    // Since value < 2^56, b0 is 0.
    // So yes, bytes 1..8 contain the 56 bits.

    out[1] = m_bytes[1];
    out[2] = m_bytes[2];
    out[3] = m_bytes[3];
    out[4] = m_bytes[4];
    out[5] = m_bytes[5];
    out[6] = m_bytes[6];
    out[7] = m_bytes[7];

    out
}

pub struct XptWriter<W: Write> {
    writer: W,
    domain: String,
    vars: Vec<(String, XptVarType)>,
}

impl<W: Write> XptWriter<W> {
    pub fn new(writer: W, domain: &str) -> Self {
        Self {
            writer,
            domain: domain.to_uppercase(),
            vars: Vec::new(),
        }
    }

    pub fn write_header(&mut self, variables: &[(&str, &str)]) -> io::Result<()> {
        let now = Utc::now();
        let date_str = format!("{:02}{:02}{:02}", now.day(), now.month(), now.year() % 100);

        // Store variable types for later use in write_row
        self.vars = variables.iter().map(|(n, t)| {
            let vt = if t.eq_ignore_ascii_case("Num") {
                XptVarType::Numeric
            } else {
                XptVarType::Character
            };
            (n.to_string(), vt)
        }).collect();

        // 1. Library Header (Standard SAS Header)
        self.write_record(&format!("HEADER RECORD*******LIBRARY HEADER RECORD!!!!!!!000000000000000000000000000000"))?;
        self.write_record(&format!("SAS     SAS     SASLIB  6.06    {}17:51:3900000000                          ", date_str))?;
        self.write_record(&format!("SAS     SAS     SASLIB  6.06    {}17:51:3900000000                          ", date_str))?;

        // 2. Member Header (The Dataset)
        self.write_record(&format!("HEADER RECORD*******MEMBER  HEADER RECORD!!!!!!!000000000000000000000000000000"))?;
        self.write_record(&format!("HEADER RECORD*******DSCRPTORHEADER RECORD!!!!!!!000000000000000000000000000000"))?;
        
        let ds_name = format!("{:8}", self.domain);
        self.write_record(&format!("SAS     {}SASDATA 6.06    {}17:51:3900000000                          ", ds_name, date_str))?;

        // 3. Variable Descriptors (NAMSTR)
        self.write_record(&format!("HEADER RECORD*******NAMESTR HEADER RECORD!!!!!!!000000000000000000000000000000"))?;

        let num_vars = variables.len() as i16;
        self.write_record(&format!("{:04}    0000000000000000000000000000000000000000000000000000000000000000000000", num_vars))?;

        // Clone vars to avoid immutable borrow of self while calling mutable method
        let vars_list = self.vars.clone();
        for (name, var_type) in &vars_list {
            self.write_namestr(name, *var_type)?;
        }

        // Close headers
        self.write_record(&format!("OBS     HEADER RECORD!!!!!!!00000000000000000000000000000000000000000000000000"))?;
        Ok(())
    }

    /// Writes a single variable definition (NAMESTR)
    fn write_namestr(&mut self, name: &str, var_type: XptVarType) -> io::Result<()> {
        let mut buf = [0u8; 140];

        // 0-1: Type (1=Numeric, 2=Char)
        let type_code: i16 = match var_type {
            XptVarType::Numeric => 1,
            XptVarType::Character => 2,
        };
        buf[0..2].copy_from_slice(&type_code.to_be_bytes());
        
        // 8-16: Name (8 bytes padded)
        let name_bytes = name.as_bytes();
        let len = name_bytes.len().min(8);
        buf[8..8+len].copy_from_slice(&name_bytes[0..len]);
        for i in 8+len..16 { buf[i] = 0x20; }

        // 16-20: Length of variable
        // Num = 8 bytes, Char = 200 bytes (fixed for this implementation)
        let var_len: i16 = match var_type {
            XptVarType::Numeric => 8,
            XptVarType::Character => 200,
        };
        buf[16..18].copy_from_slice(&var_len.to_be_bytes());

        // 40-80: Label (40 bytes)
        let label_len = name_bytes.len().min(40);
        buf[40..40+label_len].copy_from_slice(&name_bytes[0..label_len]);
        
        // Write as TWO 80-byte records
        self.writer.write_all(&buf[0..80])?;
        self.writer.write_all(&buf[80..140])?;
        self.writer.write_all(&[0u8; 20])?; // Padding to 160

        Ok(())
    }

    /// Writes the observation data
    pub fn write_row(&mut self, row: &[String]) -> io::Result<()> {
        if row.len() != self.vars.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Row length does not match header"));
        }

        for (i, val) in row.iter().enumerate() {
            let (_, var_type) = self.vars[i];
            
            match var_type {
                XptVarType::Numeric => {
                    let f_val = val.parse::<f64>().map_err(|e| {
                        io::Error::new(io::ErrorKind::InvalidData, format!("Invalid number '{}': {}", val, e))
                    })?;
                    let ibm_bytes = to_ibm_370(f_val);
                    self.writer.write_all(&ibm_bytes)?;
                },
                XptVarType::Character => {
                    let bytes = val.as_bytes();
                    let len = bytes.len().min(200);
                    self.writer.write_all(&bytes[0..len])?;
                    let padding = vec![0x20; 200 - len];
                    self.writer.write_all(&padding)?;
                }
            }
        }
        Ok(())
    }

    fn write_record(&mut self, text: &str) -> io::Result<()> {
        let bytes = text.as_bytes();
        if bytes.len() > 80 {
            self.writer.write_all(&bytes[0..80])?;
        } else {
            self.writer.write_all(bytes)?;
            let padding = vec![0x20; 80 - bytes.len()];
            self.writer.write_all(&padding)?;
        }
        Ok(())
    }
}

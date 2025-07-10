use std::str::FromStr;

use thiserror::Error;

// Custom errors for Bitcoin operations
#[derive(Error, Debug)]
pub enum BitcoinError {
    #[error("Invalid transaction format")]
    InvalidTransaction,
    #[error("Invalid script format")]
    InvalidScript,
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Parse error: {0}")]
    ParseError(String),
}

// Generic Point struct for Bitcoin addresses or coordinates
#[derive(Debug, Clone, PartialEq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self { Self { x, y } }
}

impl<T> FromStr for Point<T>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    type Err = BitcoinError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x_str, y_str) = s
            .trim()
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .and_then(|s| s.split_once(','))
            .ok_or_else(|| {
                BitcoinError::ParseError(format!(
                    "Invalid point format '{s}', expected '(x,y)'"
                ))
            })?;

        let (x_str, y_str) = (x_str.trim(), y_str.trim());

        let x = x_str.parse::<T>().map_err(|e| {
            BitcoinError::ParseError(format!("Failed to parse x coordinate '{x_str}': {e}"))
        })?;

        let y = y_str.parse::<T>().map_err(|e| {
            BitcoinError::ParseError(format!("Failed to parse y coordinate '{y_str}': {e}"))
        })?;

        Ok(Point { x, y })
    }
}

// Custom serialization for Bitcoin transaction
pub trait BitcoinSerialize {
    fn serialize(&self) -> Vec<u8> { todo!() }
}

// Legacy Bitcoin transaction
#[derive(Debug, Clone)]
pub struct LegacyTransaction {
    pub version:   i32,
    pub inputs:    Vec<TxInput>,
    pub outputs:   Vec<TxOutput>,
    pub lock_time: u32,
}

impl LegacyTransaction {
    pub fn builder() -> LegacyTransactionBuilder { todo!() }
}

// Transaction builder
pub struct LegacyTransactionBuilder {
    pub version:   i32,
    pub inputs:    Vec<TxInput>,
    pub outputs:   Vec<TxOutput>,
    pub lock_time: u32,
}

impl Default for LegacyTransactionBuilder {
    fn default() -> Self {
        Self {
            version:   1,
            inputs:    Vec::with_capacity(1),
            outputs:   Vec::with_capacity(0),
            lock_time: 0,
        }
    }
}

impl LegacyTransactionBuilder {
    pub fn new() -> Self { Self::default() }

    pub fn version(mut self, version: i32) -> Self {
        self.version = version;
        self
    }

    pub fn add_input(mut self, input: TxInput) -> Self {
        self.inputs.push(input);
        self
    }

    pub fn add_output(mut self, output: TxOutput) -> Self {
        self.outputs.push(output);
        self
    }

    pub fn lock_time(mut self, lock_time: u32) -> Self {
        self.lock_time = lock_time;
        self
    }

    pub fn build(self) -> LegacyTransaction {
        LegacyTransaction {
            version:   self.version,
            inputs:    self.inputs,
            outputs:   self.outputs,
            lock_time: self.lock_time,
        }
    }
}

// Transaction components
#[derive(Debug, Clone)]
pub struct TxInput {
    pub previous_output: OutPoint,
    pub script_sig:      Vec<u8>,
    pub sequence:        u32,
}

#[derive(Debug, Clone)]
pub struct TxOutput {
    pub value:         u64, // in satoshis
    pub script_pubkey: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct OutPoint {
    pub txid: [u8; 32],
    pub vout: u32,
}

// Simple CLI argument parser
pub fn parse_cli_args(args: &[String]) -> Result<CliCommand, BitcoinError> {
    match args {
        [] => Err(BitcoinError::ParseError("No command provided".to_string())),
        [cmd] if cmd == "balance" => Ok(CliCommand::Balance),
        [cmd, amount_str, address] if cmd == "send" => {
            let amount = amount_str
                .parse::<u64>()
                .map_err(|_| BitcoinError::ParseError("Invalid amount format".to_string()))?;
            Ok(CliCommand::Send {
                amount,
                address: address.clone(),
            })
        },
        [cmd, ..] => Err(BitcoinError::ParseError(format!("Unknown command: {cmd}"))),
    }
}

pub enum CliCommand {
    Send { amount: u64, address: String },
    Balance,
}

// Decoding legacy transaction
impl TryFrom<&[u8]> for LegacyTransaction {
    type Error = BitcoinError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // Minimum length is 16 bytes (4 version + 4 inputs count + 4 outputs count + 4
        // lock_time)
        if data.len() < 16 {
            return Err(BitcoinError::InvalidTransaction);
        }

        let version = read_i32_le(data, 0)?;
        let inputs_count = read_u32_le(data, 4)?;
        let outputs_count = read_u32_le(data, 8)?;
        let lock_time = read_u32_le(data, 12)?;

        // For this simplified implementation, we'll create vectors with proper capacity
        // based on the parsed counts from the transaction data
        let mut inputs = Vec::new();
        inputs.reserve_exact(inputs_count as usize);

        let mut outputs = Vec::new();
        outputs.reserve_exact(outputs_count as usize);

        Ok(Self {
            version,
            inputs,
            outputs,
            lock_time,
        })
    }
}

fn read_u32_le(data: &[u8], offset: usize) -> Result<u32, BitcoinError> {
    if offset + 4 > data.len() {
        return Err(BitcoinError::InvalidTransaction);
    }
    Ok(u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]))
}

// Helper function to read i32 from bytes in little-endian format
fn read_i32_le(data: &[u8], offset: usize) -> Result<i32, BitcoinError> {
    if offset + 4 > data.len() {
        return Err(BitcoinError::InvalidTransaction);
    }
    Ok(i32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]))
}

// Custom serialization for transaction
impl BitcoinSerialize for LegacyTransaction {
    fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(8);

        result.extend_from_slice(&self.version.to_le_bytes());
        result.extend_from_slice(&self.lock_time.to_le_bytes());

        result
    }
}

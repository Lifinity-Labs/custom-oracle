//! State transition types
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use enum_dispatch::enum_dispatch;
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

/// Trait representing access to program state across all versions
#[enum_dispatch]
pub trait OracleState {
    /// magic number.
    fn magic(&self) -> u32;
    /// program version.
    fn version(&self) -> u32;
    /// account type.
    fn acctype(&self) -> u32;
    /// account size.
    fn size(&self) -> u32;
    /// price type.
    fn price_type(&self) -> u32;
    /// price exponent.
    fn exponent(&self) -> i32;
    /// number of component prices.
    fn num_component_prices(&self) -> u32;
    /// number of quoters that make up aggregate.
    fn num_quoters(&self) -> u32;
    /// slot of last valid aggregate price.
    fn last_slot(&self) -> u64;
    /// valid slot-time of agg.
    fn valid_slot(&self) -> u64;
    /// exponentially moving average price.
    fn ema_price_value(&self) -> u64;
    /// numerator state for moving average price.
    fn ema_price_numerator(&self) -> u64;
    /// denominator state for moving average price.
    fn ema_price_denominator(&self) -> u64;
    /// exponentially moving average confidence interval.
    fn ema_confidence_value(&self) -> u64;
    /// numerator state for moving average confidence.
    fn ema_confidence_numerator(&self) -> u64;
    /// denominator state for moving average confidence.
    fn ema_confidence_denominator(&self) -> u64;
    /// unix timestamp of aggregate price.
    fn timestamp(&self) -> i64;
    /// min publishers for valid price.
    fn min_publishers(&self) -> u8;
    /// space for future derived values.
    fn drv2(&self) -> i8;
    /// space for future derived values.
    fn drv3(&self) -> i16;
    /// space for future derived values.
    fn drv4(&self) -> i32;
    /// product account key.
    fn product_account_key(&self) -> &Pubkey;
    /// next Price account in linked list.
    fn next_price_account_key(&self) -> &Pubkey;
    /// valid slot of previous update.
    fn previous_slot(&self) -> u64;
    /// aggregate price of previous update with TRADING status.
    fn previous_price_component(&self) -> i64;
    /// confidence interval of previous update with TRADING status.
    fn previous_confidence_component(&self) -> u64;
    /// unix timestamp of previous aggregate with TRADING status.
    fn previous_timestamp(&self) -> i64;
    /// the current price.
    fn price_component(&self) -> i64;
    /// confidence interval around the price.
    fn confidence_component(&self) -> u64;
    /// status of price.
    fn status(&self) -> u32;
    /// notification of any corporate action.
    fn corporate_action(&self) -> u32;
    /// publish slot.
    fn publish_slot(&self) -> u64;
    /// price components one per quoter.
    fn buffer(&self) -> [u128;192];
}

/// All versions of OracleState
#[enum_dispatch(OracleState)]
pub enum OracleVersion {
    /// Latest version, used for all new oracle
    OracleV1,
}

/// OracleVersion does not implement program_pack::Pack because there are size
/// checks on pack and unpack that would break backwards compatibility, so
/// special implementations are provided here
impl OracleVersion {
    /// Size of the latest version of the OracleState
    pub const LATEST_LEN: usize = 1 + OracleV1::LEN; // add one for the version enum

    /// Pack a oracle into a byte array, based on its version
    pub fn pack(src: Self, dst: &mut [u8]) -> Result<(), ProgramError> {
        match src {
            Self::OracleV1(oracle_info) => {
                dst[0] = 1;
                OracleV1::pack(oracle_info, &mut dst[0..])
            }
        }
    }

    /// Unpack the oracle account based on its version, returning the result as a
    /// OracleState trait object
    pub fn unpack(input: &[u8]) -> Result<Box<dyn OracleState>, ProgramError> {
        // let (&version, rest) = input;
        let version = 2;
        match version {
            2 => Ok(Box::new(OracleV1::unpack(input)?)),
            _ => Err(ProgramError::UninitializedAccount),
        }
    }
}

/// Program states.
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct OracleV1 {
    /// magic number.
    pub magic: u32,
    /// program version.
    pub version: u32,
    /// account type.
    pub acctype: u32,
    /// account size.
    pub size: u32,
    /// price type.
    pub price_type: u32,
    /// price exponent.
    pub exponent: i32,
    /// number of component prices.
    pub num_component_prices: u32,
    /// number of quoters that make up aggregate.
    pub num_quoters: u32,
    /// slot of last valid aggregate price.
    pub last_slot: u64,
    /// valid slot-time of agg.
    pub valid_slot: u64,
    /// exponentially moving average price.
    pub ema_price_value: u64,
    /// numerator state for moving average price.
    pub ema_price_numerator: u64,
    /// denominator state for moving average price.
    pub ema_price_denominator: u64,
    /// exponentially moving average confidence interval.
    pub ema_confidence_value: u64,
    /// numerator state for moving average confidence.
    pub ema_confidence_numerator: u64,
    /// denominator state for moving average confidence.
    pub ema_confidence_denominator: u64,
    /// unix timestamp of aggregate price.
    pub timestamp: i64,
    /// min publishers for valid price.
    pub min_publishers: u8,
    /// space for future derived values.
    pub drv2: i8,
    /// space for future derived values.
    pub drv3: i16,
    /// space for future derived values.
    pub drv4: i32,
    /// product account key.
    pub product_account_key: Pubkey,
    /// next Price account in linked list.
    pub next_price_account_key: Pubkey,
    /// valid slot of previous update.
    pub previous_slot: u64,
    /// aggregate price of previous update with TRADING status.
    pub previous_price_component: i64,
    /// confidence interval of previous update with TRADING status.
    pub previous_confidence_component: u64,
    /// unix timestamp of previous aggregate with TRADING status.
    pub previous_timestamp: i64,
    /// the current price.
    pub price_component: i64,
    /// confidence interval around the price.
    pub confidence_component: u64,
    /// status of price.
    pub status: u32,
    /// notification of any corporate action.
    pub corporate_action: u32,
    /// publish slot.
    pub publish_slot: u64,
    /// price components one per quoter.
    pub buffer: [u128;192],
}

impl OracleState for OracleV1 {
    fn magic(&self) -> u32 {
        self.magic
    }

    fn version(&self) -> u32 {
        self.version
    }

    fn acctype(&self) -> u32 {
        self.acctype
    }
    
    fn size(&self) -> u32 {
        self.size
    }
    
    fn price_type(&self) -> u32 {
        self.price_type
    }

    fn exponent(&self) -> i32 {
        self.exponent
    }

    fn num_component_prices(&self) -> u32 {
        self.num_component_prices
    }

    fn num_quoters(&self) -> u32 {
        self.num_quoters
    }

    fn last_slot(&self) -> u64 {
        self.last_slot
    }

    fn valid_slot(&self) -> u64 {
        self.valid_slot
    }

    fn ema_price_value(&self) -> u64 {
        self.ema_price_value
    }

    fn ema_price_numerator(&self) -> u64 {
        self.ema_price_numerator
    }

    fn ema_price_denominator(&self) -> u64 {
        self.ema_price_denominator
    }

    fn ema_confidence_value(&self) -> u64 {
        self.ema_confidence_value
    }

    fn ema_confidence_numerator(&self) -> u64 {
        self.ema_confidence_numerator
    }

    fn ema_confidence_denominator(&self) -> u64 {
        self.ema_confidence_denominator
    }

    fn timestamp(&self) -> i64 {
        self.timestamp
    }

    fn min_publishers(&self) -> u8 {
        self.min_publishers
    }

    fn drv2(&self) -> i8 {
        self.drv2
    }

    fn drv3(&self) -> i16 {
        self.drv3
    }

    fn drv4(&self) -> i32 {
        self.drv4
    }

    fn product_account_key(&self) -> &Pubkey {
        &self.product_account_key
    }

    fn next_price_account_key(&self) -> &Pubkey {
        &self.next_price_account_key
    }

    fn previous_slot(&self) -> u64 {
        self.previous_slot
    }

    fn previous_price_component(&self) -> i64 {
        self.previous_price_component
    }

    fn previous_confidence_component(&self) -> u64 {
        self.previous_confidence_component
    }

    fn previous_timestamp(&self) -> i64 {
        self.previous_timestamp
    }

    fn price_component(&self) -> i64 {
        self.price_component
    }

    fn confidence_component(&self) -> u64 {
        self.confidence_component
    }

    fn status(&self) -> u32 {
        self.status
    }

    fn corporate_action(&self) -> u32 {
        self.corporate_action
    }

    fn publish_slot(&self) -> u64 {
        self.publish_slot
    }

    fn buffer(&self) -> [u128;192] {
        self.buffer
    }
}

impl Sealed for OracleV1 {}

impl IsInitialized for OracleV1 {
    fn is_initialized(&self) -> bool {
        false
    }
}

impl Pack for OracleV1 {
    const LEN: usize = 3312;
    
    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, 3312];
        let (
            magic,
            version,
            acctype,
            size,
            price_type,
            exponent,
            num_component_prices,
            num_quoters,
            last_slot,
            valid_slot,
            ema_price_value,
            ema_price_numerator,
            ema_price_denominator,
            ema_confidence_value,
            ema_confidence_numerator,
            ema_confidence_denominator,
            timestamp,
            min_publishers,
            drv2,
            drv3,
            drv4,
            product_account_key,
            next_price_account_key,
            previous_slot,
            previous_price_component,
            previous_confidence_component,
            previous_timestamp,
            price_component,
            confidence_component,
            status,
            corporate_action,
            publish_slot,
            buffer,
        ) = mut_array_refs![output, 4, 4, 4, 4, 4, 4, 4, 4, 8, 8, 8, 8, 8, 8, 8, 8, 8, 1, 1, 2, 4, 32, 32, 8, 8, 8, 8, 8, 8, 4, 4, 8, 3072];
        *magic = self.magic.to_le_bytes();
        *version = self.version.to_le_bytes();
        *acctype = self.acctype.to_le_bytes();
        *size = self.size.to_le_bytes();
        *price_type = self.price_type.to_le_bytes();
        *exponent = self.exponent.to_le_bytes();
        *num_component_prices = self.num_component_prices.to_le_bytes();
        *num_quoters = self.num_quoters.to_le_bytes();
        *last_slot = self.last_slot.to_le_bytes();
        *valid_slot = self.valid_slot.to_le_bytes();
        *ema_price_value = self.ema_price_value.to_le_bytes();
        *ema_price_numerator = self.ema_price_numerator.to_le_bytes();
        *ema_price_denominator = self.ema_price_denominator.to_le_bytes();
        *ema_confidence_value = self.ema_confidence_value.to_le_bytes();
        *ema_confidence_numerator = self.ema_confidence_numerator.to_le_bytes();
        *ema_confidence_denominator = self.ema_confidence_denominator.to_le_bytes();
        *timestamp = self.timestamp.to_le_bytes();
        *min_publishers = self.min_publishers.to_le_bytes();
        *drv2 = self.drv2.to_le_bytes();
        *drv3 = self.drv3.to_le_bytes();
        *drv4 = self.drv4.to_le_bytes();
        product_account_key.copy_from_slice(self.product_account_key.as_ref());
        next_price_account_key.copy_from_slice(self.next_price_account_key.as_ref());
        *previous_slot = self.previous_slot.to_le_bytes();
        *previous_price_component = self.previous_price_component.to_le_bytes();
        *previous_confidence_component = self.previous_confidence_component.to_le_bytes();
        *previous_timestamp = self.previous_timestamp.to_le_bytes();
        *price_component = self.price_component.to_le_bytes();
        *confidence_component = self.confidence_component.to_le_bytes();
        *status = self.status.to_le_bytes();
        *corporate_action = self.corporate_action.to_le_bytes();
        *publish_slot = self.publish_slot.to_le_bytes();
        *buffer = [0;3072];
    }

    /// Unpacks a byte buffer into a [OracleV1](struct.OracleV1.html).
    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, 3312];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            magic,
            version,
            acctype,
            size,
            price_type,
            exponent,
            num_component_prices,
            num_quoters,
            last_slot,
            valid_slot,
            ema_price_value,
            ema_price_numerator,
            ema_price_denominator,
            ema_confidence_value,
            ema_confidence_numerator,
            ema_confidence_denominator,
            timestamp,
            min_publishers,
            drv2,
            drv3,
            drv4,
            product_account_key,
            next_price_account_key,
            previous_slot,
            previous_price_component,
            previous_confidence_component,
            previous_timestamp,
            price_component,
            confidence_component,
            status,
            corporate_action,
            publish_slot,
            _buffer,
        ) = array_refs![input, 4, 4, 4, 4, 4, 4, 4, 4, 8, 8, 8, 8, 8, 8, 8, 8, 8, 1, 1, 2, 4, 32, 32, 8, 8, 8, 8, 8, 8, 4, 4, 8, 3072]; 
        Ok(Self {
            magic: u32::from_le_bytes(*magic),
            version: u32::from_le_bytes(*version),
            acctype: u32::from_le_bytes(*acctype),
            size: u32::from_le_bytes(*size),
            price_type: u32::from_le_bytes(*price_type),
            exponent: i32::from_le_bytes(*exponent),
            num_component_prices: u32::from_le_bytes(*num_component_prices),
            num_quoters: u32::from_le_bytes(*num_quoters),
            last_slot: u64::from_le_bytes(*last_slot),
            valid_slot: u64::from_le_bytes(*valid_slot),
            ema_price_value: u64::from_le_bytes(*ema_price_value),
            ema_price_numerator: u64::from_le_bytes(*ema_price_numerator),
            ema_price_denominator: u64::from_le_bytes(*ema_price_denominator),
            ema_confidence_value: u64::from_le_bytes(*ema_confidence_value),
            ema_confidence_numerator: u64::from_le_bytes(*ema_confidence_numerator),
            ema_confidence_denominator: u64::from_le_bytes(*ema_confidence_denominator),
            timestamp: i64::from_le_bytes(*timestamp),
            min_publishers: u8::from_le_bytes(*min_publishers),
            drv2: i8::from_le_bytes(*drv2),
            drv3: i16::from_le_bytes(*drv3),
            drv4: i32::from_le_bytes(*drv4),
            product_account_key: Pubkey::new_from_array(*product_account_key),
            next_price_account_key: Pubkey::new_from_array(*next_price_account_key),
            previous_slot: u64::from_le_bytes(*previous_slot),
            previous_price_component: i64::from_le_bytes(*previous_price_component),
            previous_confidence_component: u64::from_le_bytes(*previous_confidence_component),
            previous_timestamp: i64::from_le_bytes(*previous_timestamp),
            price_component: i64::from_le_bytes(*price_component),
            confidence_component: u64::from_le_bytes(*confidence_component),
            status: u32::from_le_bytes(*status),
            corporate_action: u32::from_le_bytes(*corporate_action),
            publish_slot: u64::from_le_bytes(*publish_slot),
            buffer: [0;192]
        })
    }
}

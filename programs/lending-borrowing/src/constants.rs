use anchor_lang::prelude::*;

pub const ANCHOR_DISCRIMINATOR: usize = 8;

// See https://pyth.network/developers/price-feed-ids for all available IDs.
#[constant]
pub const SOL_USD_FEED_ID: &str = "7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE";

#[constant]
pub const USDC_USD_FEED_ID: &str = "Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX";

// get_price_no_older_than will fail if the price update is more than 30 seconds old (pyth oracle)
#[constant]
pub const MAX_AGE: u64 = 30;

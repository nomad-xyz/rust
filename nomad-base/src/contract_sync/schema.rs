use crate::NomadDB;
use color_eyre::Result;
use nomad_core::db::DbError;

static UPDATES_LAST_BLOCK_END: &str = "updates_last_block";
static MESSAGES_LAST_BLOCK_END: &str = "messages_last_block";

pub(crate) trait CommonContractSyncDB {
    fn store_update_latest_block_end(&self, latest_block: u32) -> Result<(), DbError>;
    fn retrieve_update_latest_block_end(&self) -> Option<u32>;
}

pub(crate) trait HomeContractSyncDB {
    fn store_message_latest_block_end(&self, latest_block: u32) -> Result<(), DbError>;
    fn retrieve_message_latest_block_end(&self) -> Option<u32>;
}

impl CommonContractSyncDB for NomadDB {
    fn store_update_latest_block_end(&self, latest_block: u32) -> Result<(), DbError> {
        self.store_encodable("", UPDATES_LAST_BLOCK_END, &latest_block)
    }

    fn retrieve_update_latest_block_end(&self) -> Option<u32> {
        self.retrieve_decodable("", UPDATES_LAST_BLOCK_END)
            .expect("db failure")
    }
}

impl HomeContractSyncDB for NomadDB {
    fn store_message_latest_block_end(&self, latest_block: u32) -> Result<(), DbError> {
        self.store_encodable("", MESSAGES_LAST_BLOCK_END, &latest_block)
    }

    fn retrieve_message_latest_block_end(&self) -> Option<u32> {
        self.retrieve_decodable("", MESSAGES_LAST_BLOCK_END)
            .expect("db failure")
    }
}

use candid::{decode_one, encode_args, utils::ArgumentEncoder, CandidType, Principal};
use pocket_ic::PocketIc;
use serde::de::DeserializeOwned;

pub struct CanisterClient<'a> {
    pub pic: &'a PocketIc,
    pub canister_id: Principal,
    caller: Principal,
}

impl<'a> CanisterClient<'a> {
    pub fn new(pic: &'a PocketIc, canister_id: Principal) -> Self {
        Self {
            pic,
            canister_id,
            caller: Principal::anonymous(),
        }
    }
    pub fn set_caller(&mut self, caller: Principal) -> &mut Self {
        self.caller = caller;
        self
    }

    pub fn raw_update(&self, method_name: &str, encoded_args: Vec<u8>) -> Result<Vec<u8>, String> {
        self.pic
            .update_call(self.canister_id, self.caller, method_name, encoded_args)
            .map_err(|e| format!("Failed to call {}: {:?}", method_name, e))
    }
    pub fn raw_query(&self, method_name: &str, encoded_args: Vec<u8>) -> Result<Vec<u8>, String> {
        self.pic
            .query_call(self.canister_id, self.caller, method_name, encoded_args)
            .map_err(|e| format!("Failed to call {}: {:?}", method_name, e))
    }
    pub fn update<Args, R>(&self, method_name: &str, args: Args) -> Result<R, String>
    where
        Args: ArgumentEncoder,
        R: CandidType + DeserializeOwned,
    {
        let result = self
            .pic
            .update_call(
                self.canister_id,
                self.caller,
                method_name,
                encode_args(args).map_err(|e| format!("Failed to encode args: {}", e))?,
            )
            .map_err(|e| format!("Failed to call {}: {:?}", method_name, e))?;

        decode_one(&result).map_err(|e| format!("Failed to decode response: {}", e))
    }

    pub fn query<Args, R>(&self, method_name: &str, args: Args) -> Result<R, String>
    where
        Args: ArgumentEncoder,
        R: CandidType + DeserializeOwned,
    {
        let result = self
            .pic
            .query_call(
                self.canister_id,
                self.caller,
                method_name,
                encode_args(args).map_err(|e| format!("Failed to encode args: {}", e))?,
            )
            .map_err(|e| format!("Failed to call {}: {:?}", method_name, e))?;

        decode_one(&result).map_err(|e| format!("Failed to decode response: {}", e))
    }

    pub fn update_no_args<R>(&self, method_name: &str) -> Result<R, String>
    where
        R: CandidType + DeserializeOwned,
    {
        let result = self
            .pic
            .update_call(
                self.canister_id,
                self.caller,
                method_name,
                encode_args(()).map_err(|e| format!("Failed to encode args: {}", e))?,
            )
            .map_err(|e| format!("Failed to call {}: {:?}", method_name, e))?;

        decode_one(&result).map_err(|e| format!("Failed to decode response: {}", e))
    }

    pub fn query_no_args<R>(&self, method_name: &str) -> Result<R, String>
    where
        R: CandidType + DeserializeOwned,
    {
        let result = self
            .pic
            .query_call(
                self.canister_id,
                self.caller,
                method_name,
                encode_args(()).map_err(|e| format!("Failed to encode args: {}", e))?,
            )
            .map_err(|e| format!("Failed to call {}: {:?}", method_name, e))?;

        decode_one(&result).map_err(|e| format!("Failed to decode response: {}", e))
    }

    pub fn caller(&self) -> Principal {
        self.caller
    }
}

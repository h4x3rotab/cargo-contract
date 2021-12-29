// Copyright 2018-2020 Parity Technologies (UK) Ltd.
// This file is part of cargo-contract.
//
// cargo-contract is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// cargo-contract is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with cargo-contract.  If not, see <http://www.gnu.org/licenses/>.

use super::{
    runtime_api::api::contracts::events::ContractEmitted,
    transcode::{env_types, ContractMessageTranscoder, TranscoderBuilder, Value},
};
use crate::Verbosity;

use anyhow::Result;
use subxt::{self, DefaultConfig, Event, TransactionEvents};

pub fn display_events(
    result: &TransactionEvents<DefaultConfig>,
    transcoder: &ContractMessageTranscoder,
    subxt_metadata: &subxt::Metadata,
    verbosity: &Verbosity,
    indentation: bool,
) -> Result<()> {
    if matches!(verbosity, Verbosity::Quiet) {
        return Ok(());
    }

    let runtime_metadata = subxt_metadata.runtime_metadata();
    let events_transcoder = TranscoderBuilder::new(&runtime_metadata.types)
        .register_custom_type::<sp_runtime::AccountId32, _>(Some("AccountId"), env_types::AccountId)
        .register_custom_type::<sp_runtime::AccountId32, _>(None, env_types::AccountId)
        .done();

    for event in result.as_slice() {
        log::debug!("displaying event {}::{}", event.pallet, event.variant);

        let event_metadata = subxt_metadata.event(event.pallet_index, event.variant_index)?;
        let event_ident = format!("{}::{}", event.pallet, event.variant);
        let event_fields = event_metadata.variant().fields();

        // todo: print event name, print event fields per line indented, possibly display only fields we are interested in...

        let decoded_event = events_transcoder.decoder().decode_composite(
            Some(event_ident.as_str()),
            event_fields,
            &mut &event.data[..],
        )?;



        println!();

        // decode and display contract events
        if <ContractEmitted as Event>::is_event(&event.pallet, &event.variant) {
            if let Value::Map(map) = decoded_event {
                if let Some(Value::Bytes(bytes)) = map.get(&Value::String("data".into())) {
                    log::debug!("Decoding contract event bytes {:?}", bytes);
                    let contract_event = transcoder.decode_contract_event(&mut bytes.bytes())?;
                    pretty_print(contract_event, indentation)?;
                    println!()
                } else {
                    return Err(anyhow::anyhow!("ContractEmitted::data should be `Vec<u8>`"));
                }
            } else {
                return Err(anyhow::anyhow!(
                    "ContractEmitted should be a struct with named fields"
                ));
            }
        }
    }
    println!();
    Ok(())
}

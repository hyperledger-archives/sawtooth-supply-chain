// Copyright 2018 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use protobuf;
use protobuf::Message;
use protobuf::RepeatedField;

use std::collections::HashMap;

use sawtooth_sdk::processor::handler::ApplyError;
use sawtooth_sdk::processor::handler::TransactionContext;
use sawtooth_sdk::processor::handler::TransactionHandler;
use sawtooth_sdk::messages::processor::TpProcessRequest;

use messages::*;
use addressing::*;

const PROPERTY_PAGE_MAX_LENGTH: usize = 256;

#[derive(Debug, Clone)]
enum Action {
    CreateAgent(payload::CreateAgentAction),
    CreateRecord(payload::CreateRecordAction),
    FinalizeRecord(payload::FinalizeRecordAction),
    CreateRecordType(payload::CreateRecordTypeAction),
    UpdateProperties(payload::UpdatePropertiesAction),
    CreateProposal(payload::CreateProposalAction),
    AnswerProposal(payload::AnswerProposalAction),
    RevokeReporter(payload::RevokeReporterAction),
}

struct SupplyChainPayload {
    action: Action,
    timestamp: u64,
}

impl SupplyChainPayload {
    pub fn new(payload: &[u8]) -> Result<Option<SupplyChainPayload>, ApplyError> {
        let payload: payload::SCPayload = match protobuf::parse_from_bytes(payload) {
            Ok(payload) => payload,
            Err(_) => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Cannot deserialize payload",
                )))
            }
        };

        let supply_chain_action = payload.get_action();
        let action = match supply_chain_action {
            payload::SCPayload_Action::CREATE_AGENT => {
                let create_agent = payload.get_create_agent();
                if create_agent.get_name() == "" {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Agent name cannot be an empty string",
                    )));
                }
                Action::CreateAgent(create_agent.clone())
            }
            payload::SCPayload_Action::CREATE_RECORD => {
                let create_record = payload.get_create_record();
                if create_record.get_record_id() == "" {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Record id cannot be empty string",
                    )));
                }
                Action::CreateRecord(create_record.clone())
            }
            payload::SCPayload_Action::FINALIZE_RECORD => {
                Action::FinalizeRecord(payload.get_finalize_record().clone())
            }
            payload::SCPayload_Action::CREATE_RECORD_TYPE => {
                let create_record_type = payload.get_create_record_type();
                if create_record_type.get_name() == "" {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Record Type name cannot be an empty string",
                    )));
                };
                let properties = create_record_type.get_properties();
                if properties.len() == 0 {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Record type must have at least one property",
                    )));
                }
                for prop in properties {
                    if prop.name == "" {
                        return Err(ApplyError::InvalidTransaction(String::from(
                            "Property name cannot be an empty string",
                        )));
                    }
                }

                Action::CreateRecordType(create_record_type.clone())
            }
            payload::SCPayload_Action::UPDATE_PROPERTIES => {
                Action::UpdateProperties(payload.get_update_properties().clone())
            }
            payload::SCPayload_Action::CREATE_PROPOSAL => {
                Action::CreateProposal(payload.get_create_proposal().clone())
            }
            payload::SCPayload_Action::ANSWER_PROPOSAL => {
                Action::AnswerProposal(payload.get_answer_proposal().clone())
            }
            payload::SCPayload_Action::REVOKE_REPORTER => {
                Action::RevokeReporter(payload.get_revoke_reporter().clone())
            }
        };
        let timestamp = match payload.get_timestamp() {
            0 => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Timestamp is not set",
                )))
            }
            x => x,
        };

        Ok(Some(SupplyChainPayload {
            action: action,
            timestamp: timestamp,
        }))
    }

    pub fn get_action(&self) -> Action {
        self.action.clone()
    }

    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }
}

pub struct SupplyChainState<'a> {
    context: &'a mut TransactionContext,
}

impl<'a> SupplyChainState<'a> {
    pub fn new(context: &'a mut TransactionContext) -> SupplyChainState {
        SupplyChainState { context: context }
    }

    pub fn get_record(&mut self, record_id: &str) -> Result<Option<record::Record>, ApplyError> {
        let address = make_record_address(record_id);
        let d = self.context.get_state(&address)?;
        match d {
            Some(packed) => {
                let records: record::RecordContainer =
                    match protobuf::parse_from_bytes(packed.as_slice()) {
                        Ok(records) => records,
                        Err(_) => {
                            return Err(ApplyError::InternalError(String::from(
                                "Cannot deserialize record container",
                            )))
                        }
                    };

                for record in records.get_entries() {
                    if record.record_id == record_id {
                        return Ok(Some(record.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn set_record(
        &mut self,
        record_id: &str,
        record: record::Record,
    ) -> Result<(), ApplyError> {
        let address = make_record_address(record_id);
        let d = self.context.get_state(&address)?;
        let mut record_container = match d {
            Some(packed) => match protobuf::parse_from_bytes(packed.as_slice()) {
                Ok(records) => records,
                Err(_) => {
                    return Err(ApplyError::InternalError(String::from(
                        "Cannot deserialize record container",
                    )))
                }
            },
            None => record::RecordContainer::new(),
        };
        // remove old record if it exists and sort the records by record id
        let records = record_container.get_entries().to_vec();
        let mut index = None;
        let mut count = 0;
        for record in records.clone() {
            if record.record_id == record_id {
                index = Some(count);
                break;
            }
            count = count + 1;
        }

        match index {
            Some(x) => {
                record_container.entries.remove(x);
            }
            None => (),
        };
        record_container.entries.push(record);
        record_container
            .entries
            .sort_by_key(|r| r.clone().record_id);
        let serialized = match record_container.write_to_bytes() {
            Ok(serialized) => serialized,
            Err(_) => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot serialize record container",
                )))
            }
        };
        self.context
            .set_state(&address, serialized.as_ref())
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn get_record_type(
        &mut self,
        type_name: &str,
    ) -> Result<Option<record::RecordType>, ApplyError> {
        let address = make_record_type_address(type_name);
        let d = self.context.get_state(&address)?;
        match d {
            Some(packed) => {
                let record_types: record::RecordTypeContainer =
                    match protobuf::parse_from_bytes(packed.as_slice()) {
                        Ok(record_types) => record_types,
                        Err(_) => {
                            return Err(ApplyError::InternalError(String::from(
                                "Cannot deserialize record type container",
                            )))
                        }
                    };

                for record_type in record_types.get_entries() {
                    if record_type.name == type_name {
                        return Ok(Some(record_type.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn set_record_type(
        &mut self,
        type_name: &str,
        record_type: record::RecordType,
    ) -> Result<(), ApplyError> {
        let address = make_record_type_address(type_name);
        let d = self.context.get_state(&address)?;
        let mut record_types = match d {
            Some(packed) => match protobuf::parse_from_bytes(packed.as_slice()) {
                Ok(record_types) => record_types,
                Err(_) => {
                    return Err(ApplyError::InternalError(String::from(
                        "Cannot deserialize record container",
                    )))
                }
            },
            None => record::RecordTypeContainer::new(),
        };

        record_types.entries.push(record_type);
        record_types.entries.sort_by_key(|rt| rt.clone().name);
        let serialized = match record_types.write_to_bytes() {
            Ok(serialized) => serialized,
            Err(_) => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot serialize record type container",
                )))
            }
        };
        self.context
            .set_state(&address, serialized.as_ref())
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn get_agent(&mut self, agent_id: &str) -> Result<Option<agent::Agent>, ApplyError> {
        let address = make_agent_address(agent_id);
        let d = self.context.get_state(&address)?;
        match d {
            Some(packed) => {
                let agents: agent::AgentContainer =
                    match protobuf::parse_from_bytes(packed.as_slice()) {
                        Ok(agents) => agents,
                        Err(_) => {
                            return Err(ApplyError::InternalError(String::from(
                                "Cannot deserialize agent container",
                            )))
                        }
                    };

                for agent in agents.get_entries() {
                    if agent.public_key == agent_id {
                        return Ok(Some(agent.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn set_agent(&mut self, agent_id: &str, agent: agent::Agent) -> Result<(), ApplyError> {
        let address = make_agent_address(agent_id);
        let d = self.context.get_state(&address)?;
        let mut agents = match d {
            Some(packed) => match protobuf::parse_from_bytes(packed.as_slice()) {
                Ok(agents) => agents,
                Err(_) => {
                    return Err(ApplyError::InternalError(String::from(
                        "Cannot deserialize agent container",
                    )))
                }
            },
            None => agent::AgentContainer::new(),
        };

        agents.entries.push(agent);
        agents.entries.sort_by_key(|a| a.clone().public_key);
        let serialized = match agents.write_to_bytes() {
            Ok(serialized) => serialized,
            Err(_) => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot serialize agent container",
                )))
            }
        };
        self.context
            .set_state(&&address, serialized.as_ref())
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn get_property(
        &mut self,
        record_id: &str,
        property_name: &str,
    ) -> Result<Option<property::Property>, ApplyError> {
        let address = make_property_address(record_id, property_name, 0);
        let d = self.context.get_state(&address)?;
        match d {
            Some(packed) => {
                let properties: property::PropertyContainer =
                    match protobuf::parse_from_bytes(packed.as_slice()) {
                        Ok(properties) => properties,
                        Err(_) => {
                            return Err(ApplyError::InternalError(String::from(
                                "Cannot deserialize property container",
                            )))
                        }
                    };

                for property in properties.get_entries() {
                    if property.name == property_name {
                        return Ok(Some(property.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn set_property(
        &mut self,
        record_id: &str,
        property_name: &str,
        property: property::Property,
    ) -> Result<(), ApplyError> {
        let address = make_property_address(record_id, property_name, 0);
        let d = self.context.get_state(&address)?;
        let mut property_container = match d {
            Some(packed) => match protobuf::parse_from_bytes(packed.as_slice()) {
                Ok(properties) => properties,
                Err(_) => {
                    return Err(ApplyError::InternalError(String::from(
                        "Cannot deserialize property container",
                    )))
                }
            },
            None => property::PropertyContainer::new(),
        };
        // remove old property if it exists and sort the properties by name
        let properties = property_container.get_entries().to_vec();
        let mut index = None;
        let mut count = 0;
        for prop in properties.clone() {
            if prop.name == property_name {
                index = Some(count);
                break;
            }
            count = count + 1;
        }

        match index {
            Some(x) => {
                property_container.entries.remove(x);
            }
            None => (),
        };
        property_container.entries.push(property);
        property_container.entries.sort_by_key(|p| p.clone().name);
        let serialized = match property_container.write_to_bytes() {
            Ok(serialized) => serialized,
            Err(_) => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot serialize property container",
                )))
            }
        };
        self.context
            .set_state(&address, serialized.as_ref())
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn get_property_page(
        &mut self,
        record_id: &str,
        property_name: &str,
        page: u32,
    ) -> Result<Option<property::PropertyPage>, ApplyError> {
        let address = make_property_address(record_id, property_name, page);
        let d = self.context.get_state(&address)?;
        match d {
            Some(packed) => {
                let property_pages: property::PropertyPageContainer =
                    match protobuf::parse_from_bytes(packed.as_slice()) {
                        Ok(property_pages) => property_pages,
                        Err(_) => {
                            return Err(ApplyError::InternalError(String::from(
                                "Cannot deserialize property page container",
                            )))
                        }
                    };

                for property_page in property_pages.get_entries() {
                    if property_page.name == property_name {
                        return Ok(Some(property_page.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn set_property_page(
        &mut self,
        record_id: &str,
        property_name: &str,
        page_num: u32,
        property_page: property::PropertyPage,
    ) -> Result<(), ApplyError> {
        let address = make_property_address(record_id, property_name, page_num);
        let d = self.context.get_state(&address)?;
        let mut property_pages = match d {
            Some(packed) => match protobuf::parse_from_bytes(packed.as_slice()) {
                Ok(property_pages) => property_pages,
                Err(_) => {
                    return Err(ApplyError::InternalError(String::from(
                        "Cannot deserialize property page container",
                    )))
                }
            },
            None => property::PropertyPageContainer::new(),
        };
        // remove old property page if it exists and sort the property pages by name
        let pages = property_pages.get_entries().to_vec();
        let mut index = None;
        let mut count = 0;
        for page in pages.clone() {
            if page.name == property_name {
                index = Some(count);
                break;
            }
            count = count + 1;
        }

        match index {
            Some(x) => {
                property_pages.entries.remove(x);
            }
            None => (),
        };
        property_pages.entries.push(property_page);
        property_pages.entries.sort_by_key(|pp| pp.clone().name);
        let serialized = match property_pages.write_to_bytes() {
            Ok(serialized) => serialized,
            Err(_) => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot serialize property page container",
                )))
            }
        };
        self.context
            .set_state(&address, serialized.as_ref())
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn get_proposal_container(
        &mut self,
        record_id: &str,
        agent_id: &str,
    ) -> Result<Option<proposal::ProposalContainer>, ApplyError> {
        let address = make_proposal_address(record_id, agent_id);
        let d = self.context.get_state(&address)?;
        match d {
            Some(packed) => {
                let proposals: proposal::ProposalContainer =
                    match protobuf::parse_from_bytes(packed.as_slice()) {
                        Ok(property_pages) => property_pages,
                        Err(_) => {
                            return Err(ApplyError::InternalError(String::from(
                                "Cannot deserialize proposal container",
                            )))
                        }
                    };

                Ok(Some(proposals))
            }
            None => Ok(None),
        }
    }

    pub fn set_proposal_container(
        &mut self,
        record_id: &str,
        agent_id: &str,
        proposals: proposal::ProposalContainer,
    ) -> Result<(), ApplyError> {
        let address = make_proposal_address(record_id, agent_id);
        let serialized = match proposals.write_to_bytes() {
            Ok(serialized) => serialized,
            Err(_) => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot serialize proposal container",
                )))
            }
        };
        self.context
            .set_state(&address, serialized.as_ref())
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }
}

pub struct SupplyChainTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<String>,
}

impl SupplyChainTransactionHandler {
    pub fn new() -> SupplyChainTransactionHandler {
        SupplyChainTransactionHandler {
            family_name: "supply_chain".to_string(),
            family_versions: vec!["1.1".to_string()],
            namespaces: vec![get_supply_chain_prefix().to_string()],
        }
    }

    fn _create_agent(
        &self,
        payload: payload::CreateAgentAction,
        mut state: SupplyChainState,
        signer: &str,
        timestamp: u64,
    ) -> Result<(), ApplyError> {
        let name = payload.get_name();
        match state.get_agent(signer) {
            Ok(Some(_)) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Agent already exists: {}",
                    name
                )))
            }
            Ok(None) => (),
            Err(err) => return Err(err),
        }

        let mut new_agent = agent::Agent::new();
        new_agent.set_public_key(signer.to_string());
        new_agent.set_name(name.to_string());
        new_agent.set_timestamp(timestamp);

        state.set_agent(signer, new_agent)?;
        Ok(())
    }

    fn _create_record(
        &self,
        payload: payload::CreateRecordAction,
        mut state: SupplyChainState,
        signer: &str,
        timestamp: u64,
    ) -> Result<(), ApplyError> {
        match state.get_agent(signer) {
            Ok(Some(_)) => (),
            Ok(None) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Agent is not register: {}",
                    signer
                )))
            }
            Err(err) => return Err(err),
        }
        let record_id = payload.get_record_id();
        match state.get_record(record_id) {
            Ok(Some(_)) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Record already exists: {}",
                    record_id
                )))
            }
            Ok(None) => (),
            Err(err) => return Err(err),
        }

        let type_name = payload.get_record_type();
        let record_type = match state.get_record_type(type_name) {
            Ok(Some(record_type)) => record_type,
            Ok(None) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Record Type does not exist {}",
                    type_name
                )))
            }
            Err(err) => return Err(err),
        };

        let mut type_schemata: HashMap<&str, property::PropertySchema> = HashMap::new();
        let mut required_properties: HashMap<&str, property::PropertySchema> = HashMap::new();
        let mut provided_properties: HashMap<&str, property::PropertyValue> = HashMap::new();
        for property in record_type.get_properties() {
            type_schemata.insert(property.get_name(), property.clone());
            if property.get_required() {
                required_properties.insert(property.get_name(), property.clone());
            }
        }

        for property in payload.get_properties() {
            provided_properties.insert(property.get_name(), property.clone());
        }

        for name in required_properties.keys() {
            if !provided_properties.contains_key(name) {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Required property {} not provided",
                    name
                )));
            }
        }

        for (provided_name, provided_properties) in provided_properties.clone() {
            let required_type = match type_schemata.get(provided_name) {
                Some(required_type) => required_type.data_type,
                None => {
                    return Err(ApplyError::InvalidTransaction(format!(
                        "Provided property {} is not in schemata",
                        provided_name
                    )))
                }
            };
            let provided_type = provided_properties.data_type;
            if provided_type != required_type {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Value provided for {} is the wrong type",
                    provided_name
                )));
            };

            let is_delayed = match type_schemata.get(provided_name) {
                Some(property_schema) => property_schema.delayed,
                None => false,
            };
            if is_delayed {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Property is 'delayed', and cannot be set at record creation: {}",
                    provided_name
                )));
            };
        }
        let mut new_record = record::Record::new();
        new_record.set_record_id(record_id.to_string());
        new_record.set_record_type(type_name.to_string());
        new_record.set_field_final(false);

        let mut owner = record::Record_AssociatedAgent::new();
        owner.set_agent_id(signer.to_string());
        owner.set_timestamp(timestamp);
        new_record.owners.push(owner.clone());
        new_record.custodians.push(owner.clone());

        state.set_record(record_id, new_record)?;

        let mut reporter = property::Property_Reporter::new();
        reporter.set_public_key(signer.to_string());
        reporter.set_authorized(true);
        reporter.set_index(0);

        for (property_name, property) in type_schemata {
            let mut new_property = property::Property::new();
            new_property.set_name(property_name.to_string());
            new_property.set_record_id(record_id.to_string());
            new_property.set_data_type(property.get_data_type());
            new_property.reporters.push(reporter.clone());
            new_property.set_current_page(1);
            new_property.set_wrapped(false);
            new_property.set_number_exponent(property.get_number_exponent());
            new_property.set_enum_options(property.enum_options);
            new_property.set_struct_properties(property.struct_properties);

            state.set_property(record_id, property_name, new_property.clone())?;

            let mut new_property_page = property::PropertyPage::new();
            new_property_page.set_name(property_name.to_string());
            new_property_page.set_record_id(record_id.to_string());

            if provided_properties.contains_key(property_name) {
                let provided_property = &provided_properties[property_name];
                let reported_value = match self._make_new_reported_value(
                    0,
                    timestamp,
                    provided_property,
                    &new_property,
                ) {
                    Ok(reported_value) => reported_value,
                    Err(err) => return Err(err),
                };

                new_property_page.reported_values.push(reported_value);
            }
            state.set_property_page(record_id, property_name, 1, new_property_page)?;
        }

        Ok(())
    }

    fn _finalize_record(
        &self,
        payload: payload::FinalizeRecordAction,
        mut state: SupplyChainState,
        signer: &str,
    ) -> Result<(), ApplyError> {
        let record_id = payload.get_record_id();
        let final_record = match state.get_record(record_id) {
            Ok(Some(final_record)) => final_record,
            Ok(None) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Record does not exist: {}",
                    record_id
                )))
            }
            Err(err) => return Err(err),
        };
        let owner = match final_record.owners.last() {
            Some(x) => x,
            None => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Owner was not found",
                )))
            }
        };
        let custodian = match final_record.custodians.last() {
            Some(x) => x,
            None => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Custodian was not found",
                )))
            }
        };

        if owner.agent_id != signer || custodian.agent_id != signer {
            return Err(ApplyError::InvalidTransaction(format!(
                "Must be owner and custodian to finalize record"
            )));
        }
        if final_record.get_field_final() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Record is already final: {}",
                record_id
            )));
        }

        let mut record_clone = final_record.clone();
        record_clone.set_field_final(true);
        state.set_record(record_id, record_clone)?;

        Ok(())
    }

    fn _create_record_type(
        &self,
        payload: payload::CreateRecordTypeAction,
        mut state: SupplyChainState,
        signer: &str,
    ) -> Result<(), ApplyError> {
        match state.get_agent(signer) {
            Ok(Some(_)) => (),
            Ok(None) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Agent is not register: {}",
                    signer
                )))
            }
            Err(err) => return Err(err),
        }
        let name = payload.get_name();
        let mut provided_properties: HashMap<&str, property::PropertySchema> = HashMap::new();
        for property in payload.get_properties() {
            provided_properties.insert(property.get_name(), property.clone());
        }
        match state.get_record_type(name) {
            Ok(Some(_)) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Record type already exists: {}",
                    signer
                )))
            }
            Ok(None) => (),
            Err(err) => return Err(err),
        }
        let mut record_type = record::RecordType::new();
        record_type.set_name(name.to_string());
        record_type.set_properties(RepeatedField::from_vec(payload.get_properties().to_vec()));

        state.set_record_type(name, record_type)?;

        Ok(())
    }

    fn _update_properties(
        &self,
        payload: payload::UpdatePropertiesAction,
        mut state: SupplyChainState,
        signer: &str,
        timestamp: u64,
    ) -> Result<(), ApplyError> {
        let record_id = payload.get_record_id();
        let update_record = match state.get_record(record_id) {
            Ok(Some(update_record)) => update_record,
            Ok(None) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Record does not exist: {}",
                    record_id
                )))
            }
            Err(err) => return Err(err),
        };

        if update_record.get_field_final() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Record is final: {}",
                record_id
            )));
        }

        let updates = payload.get_properties();

        for update in updates {
            let name = update.get_name();
            let data_type = update.get_data_type();

            let mut prop = match state.get_property(record_id, name) {
                Ok(Some(prop)) => prop,
                Ok(None) => {
                    return Err(ApplyError::InvalidTransaction(format!(
                        "Record does not have provided poperty: {}",
                        name
                    )))
                }
                Err(err) => return Err(err),
            };

            let mut allowed = false;
            let mut reporter_index = 0;
            for reporter in prop.get_reporters() {
                if reporter.get_public_key() == signer && reporter.get_authorized() {
                    allowed = true;
                    reporter_index = reporter.get_index();
                    break;
                }
            }
            if !allowed {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Reporter is not authorized: {}",
                    signer
                )));
            }

            if data_type != prop.data_type {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Update has wrong type: {:?} != {:?}",
                    data_type, prop.data_type
                )));
            }

            let page_number = prop.get_current_page();
            let mut page = match state.get_property_page(record_id, name, page_number) {
                Ok(Some(page)) => page,
                Ok(None) => {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Property page does not exist",
                    )))
                }
                Err(err) => return Err(err),
            };

            let reported_value = match self._make_new_reported_value(
                reporter_index,
                timestamp,
                update,
                &prop,
            ) {
                Ok(reported_value) => reported_value,
                Err(err) => return Err(err),
            };
            page.reported_values.push(reported_value);
            page.reported_values
                .sort_by_key(|rv| (rv.clone().timestamp, rv.clone().reporter_index));
            state.set_property_page(record_id, name, page_number, page.clone())?;
            if page.reported_values.len() >= PROPERTY_PAGE_MAX_LENGTH {
                let mut new_page_number = page_number + 1;
                if page_number + 1 <= PROPERTY_PAGE_MAX_LENGTH as u32 {
                    new_page_number = 1;
                }

                let new_page = match state.get_property_page(record_id, name, new_page_number) {
                    Ok(Some(mut new_page)) => {
                        new_page.set_reported_values(RepeatedField::from_vec(Vec::new()));
                        new_page
                    }
                    Ok(None) => {
                        let mut new_page = property::PropertyPage::new();
                        new_page.set_name(name.to_string());
                        new_page.set_record_id(record_id.to_string());
                        new_page
                    }
                    Err(err) => return Err(err),
                };
                state.set_property_page(record_id, name, new_page_number, new_page)?;

                prop.set_current_page(new_page_number);
                if new_page_number == 1 && !prop.get_wrapped() {
                    prop.set_wrapped(true);
                }
                state.set_property(record_id, name, prop)?;
            }
        }

        Ok(())
    }

    fn _create_proposal(
        &self,
        payload: payload::CreateProposalAction,
        mut state: SupplyChainState,
        signer: &str,
        timestamp: u64,
    ) -> Result<(), ApplyError> {
        let record_id = payload.record_id;
        let receiving_agent = payload.receiving_agent;
        let role = payload.role;
        let properties = payload.properties;

        match state.get_agent(signer) {
            Ok(Some(agent)) => agent,
            Ok(None) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Issuing agent does not exist: {}",
                    signer
                )))
            }
            Err(err) => return Err(err),
        };

        match state.get_agent(&receiving_agent) {
            Ok(Some(agent)) => agent,
            Ok(None) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Receiving agent does not exist: {}",
                    receiving_agent
                )))
            }
            Err(err) => return Err(err),
        };

        let mut proposals = match state.get_proposal_container(&record_id, &receiving_agent) {
            Ok(Some(proposals)) => proposals,
            Ok(None) => proposal::ProposalContainer::new(),
            Err(err) => return Err(err),
        };

        let mut open_proposals = Vec::<proposal::Proposal>::new();
        for prop in proposals.get_entries() {
            if prop.status == proposal::Proposal_Status::OPEN {
                open_proposals.push(prop.clone());
            }
        }

        for prop in open_proposals {
            if prop.get_receiving_agent() == receiving_agent && prop.get_role() == role
                && prop.get_record_id() == record_id
            {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Proposal already exists",
                )));
            }
        }

        let proposal_record = match state.get_record(&record_id) {
            Ok(Some(record)) => record,
            Ok(None) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Record does not exist: {}",
                    record_id
                )))
            }
            Err(err) => return Err(err),
        };

        if proposal_record.get_field_final() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Record is final: {}",
                record_id
            )));
        }

        if role == proposal::Proposal_Role::OWNER || role == proposal::Proposal_Role::REPORTER {
            let owner = match proposal_record.owners.last() {
                Some(owner) => owner,
                None => {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Owner not found",
                    )))
                }
            };
            if owner.get_agent_id() != signer {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Only the owner can create a proposal to change ownership",
                )));
            }
        }

        if role == proposal::Proposal_Role::CUSTODIAN {
            let custodian = match proposal_record.custodians.last() {
                Some(custodian) => custodian,
                None => {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Custodian not found",
                    )))
                }
            };

            if custodian.get_agent_id() != signer {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Only the custodian can create a proposal to change custodianship",
                )));
            }
        }

        let mut new_proposal = proposal::Proposal::new();
        new_proposal.set_record_id(record_id.to_string());
        new_proposal.set_timestamp(timestamp);
        new_proposal.set_issuing_agent(signer.to_string());
        new_proposal.set_receiving_agent(receiving_agent.to_string());
        new_proposal.set_role(role);
        new_proposal.set_properties(properties);
        new_proposal.set_status(proposal::Proposal_Status::OPEN);

        proposals.entries.push(new_proposal);
        proposals.entries.sort_by_key(|p| {
            (
                p.clone().record_id,
                p.clone().receiving_agent,
                p.clone().timestamp,
            )
        });
        state.set_proposal_container(&record_id, &receiving_agent, proposals)?;

        Ok(())
    }

    fn _answer_proposal(
        &self,
        payload: payload::AnswerProposalAction,
        mut state: SupplyChainState,
        signer: &str,
        timestamp: u64,
    ) -> Result<(), ApplyError> {
        let record_id = payload.get_record_id();
        let receiving_agent = payload.get_receiving_agent();
        let role = payload.get_role();
        let response = payload.get_response();

        let mut proposals = match state.get_proposal_container(record_id, receiving_agent) {
            Ok(Some(proposals)) => proposals,
            Ok(None) => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Proposal does not exist",
                )))
            }
            Err(err) => return Err(err),
        };

        let mut exists = false;
        let mut current_proposal = match proposals.clone().entries.last() {
            Some(current_proposal) => current_proposal.clone(),
            None => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "No open proposals found for record {} for {}",
                    record_id, receiving_agent
                )))
            }
        };

        let mut proposal_index = 0;
        let mut count = 0;

        for prop in proposals.get_entries() {
            if prop.get_receiving_agent() == receiving_agent && prop.get_role() == role
                && prop.get_record_id() == record_id
                && prop.status == proposal::Proposal_Status::OPEN
            {
                current_proposal = prop.clone();
                exists = true;
                proposal_index = count;
                break;
            }
            count = count + 1;
        }

        if !exists {
            return Err(ApplyError::InvalidTransaction(format!(
                "No open proposals found for record {} for {}",
                record_id, receiving_agent
            )));
        }

        match response {
            payload::AnswerProposalAction_Response::CANCEL => {
                if current_proposal.get_issuing_agent() != signer {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Only the issuing agent can cancel a proposal",
                    )));
                }
                current_proposal.status = proposal::Proposal_Status::CANCELED;
            }
            payload::AnswerProposalAction_Response::REJECT => {
                if current_proposal.get_receiving_agent() != signer {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Only the receiving agent can reject a proposal",
                    )));
                }
                current_proposal.status = proposal::Proposal_Status::REJECTED;
            }
            payload::AnswerProposalAction_Response::ACCEPT => {
                if current_proposal.get_receiving_agent() != signer {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Only the receiving agent can Accept a proposal",
                    )));
                };

                let mut proposal_record = match state.get_record(record_id) {
                    Ok(Some(record)) => record,
                    Ok(None) => {
                        return Err(ApplyError::InvalidTransaction(format!(
                            "Record in proposal does not exist: {}",
                            record_id
                        )))
                    }
                    Err(err) => return Err(err),
                };

                let owner = match proposal_record.clone().owners.last() {
                    Some(owner) => owner.clone(),
                    None => {
                        return Err(ApplyError::InvalidTransaction(String::from(
                            "Owner not found",
                        )))
                    }
                };

                let custodian = match proposal_record.clone().custodians.last() {
                    Some(custodian) => custodian.clone(),
                    None => {
                        return Err(ApplyError::InvalidTransaction(String::from(
                            "Custodian not found",
                        )))
                    }
                };

                match role {
                    proposal::Proposal_Role::OWNER => {
                        if owner.get_agent_id() != current_proposal.get_issuing_agent() {
                            current_proposal.status = proposal::Proposal_Status::CANCELED;
                            info!("Record owner does not match the issuing agent of the proposal");
                            // remove old proposal and replace with new one
                            proposals.entries.remove(proposal_index);
                            proposals.entries.push(current_proposal);
                            proposals.entries.sort_by_key(|p| {
                                (
                                    p.clone().record_id,
                                    p.clone().receiving_agent,
                                    p.clone().timestamp,
                                )
                            });
                            state.set_proposal_container(&record_id, &receiving_agent, proposals)?;
                            return Ok(());
                        }

                        let mut new_owner = record::Record_AssociatedAgent::new();
                        new_owner.set_agent_id(receiving_agent.to_string());
                        new_owner.set_timestamp(timestamp);
                        proposal_record.owners.push(new_owner);
                        state.set_record(record_id, proposal_record.clone())?;

                        let record_type =
                            match state.get_record_type(proposal_record.get_record_type()) {
                                Ok(Some(record_type)) => record_type,
                                Ok(None) => {
                                    return Err(ApplyError::InvalidTransaction(format!(
                                        "RecordType does not exist: {}",
                                        proposal_record.get_record_type()
                                    )))
                                }
                                Err(err) => return Err(err),
                            };

                        for prop_schema in record_type.get_properties() {
                            let mut prop =
                                match state.get_property(record_id, prop_schema.get_name()) {
                                    Ok(Some(prop)) => prop,
                                    Ok(None) => {
                                        return Err(ApplyError::InvalidTransaction(String::from(
                                            "Property does not exist",
                                        )))
                                    }
                                    Err(err) => return Err(err),
                                };

                            let mut authorized = false;
                            let mut new_reporters: Vec<
                                property::Property_Reporter,
                            > = Vec::new();
                            let temp_prob = prop.clone();
                            let reporters = temp_prob.get_reporters();
                            for reporter in reporters {
                                if reporter.get_public_key() == owner.get_agent_id() {
                                    let mut new_reporter = reporter.clone();
                                    new_reporter.set_authorized(false);
                                    new_reporters.push(new_reporter);
                                } else if reporter.get_public_key() == receiving_agent {
                                    let mut new_reporter = reporter.clone();
                                    new_reporter.set_authorized(true);
                                    authorized = true;
                                    new_reporters.push(new_reporter);
                                } else {
                                    new_reporters.push(reporter.clone());
                                }
                            }

                            if !authorized {
                                let mut reporter = property::Property_Reporter::new();
                                reporter.set_public_key(receiving_agent.to_string());
                                reporter.set_authorized(true);
                                reporter.set_index(prop.reporters.len() as u32);
                                new_reporters.push(reporter);
                            }

                            prop.set_reporters(RepeatedField::from_vec(new_reporters));
                            state.set_property(record_id, prop.get_name(), prop.clone())?;
                        }
                        current_proposal.status = proposal::Proposal_Status::ACCEPTED;
                    }
                    proposal::Proposal_Role::CUSTODIAN => {
                        if custodian.get_agent_id() != current_proposal.get_issuing_agent() {
                            current_proposal.status = proposal::Proposal_Status::CANCELED;
                            info!(
                                "Record custodian does not match the issuing agent of the proposal"
                            );
                            // remove old proposal and replace with new one
                            proposals.entries.remove(proposal_index);
                            proposals.entries.push(current_proposal.clone());
                            proposals.entries.sort_by_key(|p| {
                                (
                                    p.clone().record_id,
                                    p.clone().receiving_agent,
                                    p.clone().timestamp,
                                )
                            });
                            state.set_proposal_container(
                                &record_id,
                                &receiving_agent,
                                proposals.clone(),
                            )?;
                        }

                        let mut new_custodian = record::Record_AssociatedAgent::new();
                        new_custodian.set_agent_id(receiving_agent.to_string());
                        new_custodian.set_timestamp(timestamp);
                        proposal_record.custodians.push(new_custodian.clone());
                        state.set_record(record_id, proposal_record)?;
                        current_proposal.status = proposal::Proposal_Status::ACCEPTED;
                    }
                    proposal::Proposal_Role::REPORTER => {
                        if owner.get_agent_id() != current_proposal.get_issuing_agent() {
                            current_proposal.status = proposal::Proposal_Status::CANCELED;
                            info!("Record owner does not match the issuing agent of the proposal");
                            // remove old proposal and replace with new one
                            proposals.entries.remove(proposal_index);
                            proposals.entries.push(current_proposal);
                            proposals.entries.sort_by_key(|p| {
                                (
                                    p.clone().record_id,
                                    p.clone().receiving_agent,
                                    p.clone().timestamp,
                                )
                            });
                            state.set_proposal_container(&record_id, &receiving_agent, proposals)?;
                            return Ok(());
                        }

                        let mut reporter = property::Property_Reporter::new();
                        reporter.set_public_key(receiving_agent.to_string());
                        reporter.set_authorized(true);

                        for prop_name in current_proposal.get_properties() {
                            let mut prop = match state.get_property(record_id, prop_name) {
                                Ok(Some(prop)) => prop,
                                Ok(None) => {
                                    return Err(ApplyError::InvalidTransaction(String::from(
                                        "Property does not exist",
                                    )))
                                }
                                Err(err) => return Err(err),
                            };
                            reporter.set_index(prop.reporters.len() as u32);
                            prop.reporters.push(reporter.clone());
                            state.set_property(record_id, prop_name, prop)?;
                        }
                        current_proposal.status = proposal::Proposal_Status::ACCEPTED;
                    }
                }
            }
        }
        // remove old proposal and replace with new one
        proposals.entries.remove(proposal_index);
        proposals.entries.push(current_proposal.clone());
        proposals.entries.sort_by_key(|p| {
            (
                p.clone().record_id,
                p.clone().receiving_agent,
                p.clone().timestamp,
            )
        });
        state.set_proposal_container(&record_id, &receiving_agent, proposals)?;

        Ok(())
    }

    fn _revoke_reporter(
        &self,
        payload: payload::RevokeReporterAction,
        mut state: SupplyChainState,
        signer: &str,
    ) -> Result<(), ApplyError> {
        let record_id = payload.get_record_id();
        let reporter_id = payload.get_reporter_id();
        let properties = payload.get_properties();

        let revoke_record = match state.get_record(record_id) {
            Ok(Some(record)) => record,
            Ok(None) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Record does not exists: {}",
                    record_id
                )))
            }
            Err(err) => return Err(err),
        };

        let owner = match revoke_record.owners.last() {
            Some(x) => x,
            None => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Owner was not found",
                )))
            }
        };

        if owner.get_agent_id() != signer {
            return Err(ApplyError::InvalidTransaction(format!(
                "Must be owner to revoke reporters"
            )));
        }

        if revoke_record.get_field_final() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Record is final: {}",
                record_id
            )));
        }

        for prop_name in properties {
            let mut prop = match state.get_property(record_id, prop_name) {
                Ok(Some(prop)) => prop,
                Ok(None) => {
                    return Err(ApplyError::InvalidTransaction(format!(
                        "Property does not exists"
                    )))
                }
                Err(err) => return Err(err),
            };

            let mut new_reporters: Vec<property::Property_Reporter> = Vec::new();
            let mut revoked = false;
            for reporter in prop.get_reporters() {
                if reporter.get_public_key() == reporter_id {
                    if !reporter.get_authorized() {
                        return Err(ApplyError::InvalidTransaction(format!(
                            "Reporter is already unauthorized."
                        )));
                    }
                    let mut unauthorized_reporter = reporter.clone();
                    unauthorized_reporter.set_authorized(false);
                    revoked = true;
                    new_reporters.push(unauthorized_reporter);
                } else {
                    new_reporters.push(reporter.clone());
                }
            }
            if !revoked {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Reporter cannot be revoked: {}",
                    reporter_id
                )));
            }
            prop.set_reporters(RepeatedField::from_vec(new_reporters));

            state.set_property(record_id, prop_name, prop)?;
        }

        Ok(())
    }

    fn _make_new_reported_value(
        &self,
        reporter_index: u32,
        timestamp: u64,
        value: &property::PropertyValue,
        property: &property::Property,
    ) -> Result<property::PropertyPage_ReportedValue, ApplyError> {
        let mut reported_value = property::PropertyPage_ReportedValue::new();
        reported_value.set_reporter_index(reporter_index);
        reported_value.set_timestamp(timestamp);

        match value.get_data_type() {
            property::PropertySchema_DataType::TYPE_UNSET => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "DataType is not set",
                )))
            }
            property::PropertySchema_DataType::BYTES => {
                reported_value.set_bytes_value(value.get_bytes_value().to_vec())
            }
            property::PropertySchema_DataType::BOOLEAN => {
                reported_value.set_boolean_value(value.get_boolean_value())
            }
            property::PropertySchema_DataType::NUMBER => {
                reported_value.set_number_value(value.get_number_value())
            }
            property::PropertySchema_DataType::STRING => {
                reported_value.set_string_value(value.get_string_value().to_string())
            }
            property::PropertySchema_DataType::ENUM => {
                let enum_name = value.get_enum_value().to_string();
                let enum_index = match property.enum_options.iter()
                    .position(|name| name == &enum_name) {
                        Some(index) => index,
                        None => {
                            return Err(ApplyError::InvalidTransaction(format!(
                                "Provided enum name is not a valid option: {}",
                                enum_name,
                            )))
                        }
                    };
                reported_value.set_enum_value(enum_index as u32)
            }
            property::PropertySchema_DataType::STRUCT => {
                match self._validate_struct_values(
                    &value.struct_values,
                    &property.struct_properties
                ) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                }

                let struct_values = RepeatedField::from_vec(value.get_struct_values().to_vec());
                reported_value.set_struct_values(struct_values)
            }
            property::PropertySchema_DataType::LOCATION => {
                reported_value.set_location_value(value.get_location_value().clone())
            }
        };
        Ok(reported_value)
    }

    fn _validate_struct_values(
        &self,
        struct_values: &RepeatedField<property::PropertyValue>,
        schema_values: &RepeatedField<property::PropertySchema>
    ) -> Result<(), ApplyError> {
        if struct_values.len() != schema_values.len() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Provided struct does not match schema length: {:?} != {:?}",
                struct_values.len(),
                schema_values.len(),
            )))
        }

        for schema in schema_values.iter() {
            let value = match struct_values.iter().find(|val| val.name == schema.name) {
                Some(val) => val,
                None => return Err(ApplyError::InvalidTransaction(format!(
                    "Provided struct missing required property from schema: {}",
                    schema.name,
                )))
            };

            if value.data_type != schema.data_type {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Struct property \"{}\" must have data type: {:?}",
                    schema.name,
                    schema.data_type,
                )))
            }

            if schema.data_type == property::PropertySchema_DataType::STRUCT {
                match self._validate_struct_values(
                    &value.struct_values,
                    &schema.struct_properties
                ) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(())
    }
}

impl TransactionHandler for SupplyChainTransactionHandler {
    fn family_name(&self) -> String {
        return self.family_name.clone();
    }

    fn family_versions(&self) -> Vec<String> {
        return self.family_versions.clone();
    }

    fn namespaces(&self) -> Vec<String> {
        return self.namespaces.clone();
    }

    fn apply(
        &self,
        request: &TpProcessRequest,
        context: &mut TransactionContext,
    ) -> Result<(), ApplyError> {
        let payload = SupplyChainPayload::new(request.get_payload());
        let payload = match payload {
            Err(e) => return Err(e),
            Ok(payload) => payload,
        };
        let payload = match payload {
            Some(x) => x,
            None => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Request must contain a payload",
                )))
            }
        };

        let signer = request.get_header().get_signer_public_key();
        let state = SupplyChainState::new(context);

        info!(
            "payload: {:?} {} {} {}",
            payload.get_action(),
            payload.get_timestamp(),
            request.get_header().get_inputs()[0],
            request.get_header().get_outputs()[0]
        );

        match payload.get_action() {
            Action::CreateAgent(agent_payload) => {
                self._create_agent(agent_payload, state, signer, payload.get_timestamp())?
            }
            Action::CreateRecord(record_payload) => {
                self._create_record(record_payload, state, signer, payload.get_timestamp())?
            }
            Action::FinalizeRecord(finalize_payload) => {
                self._finalize_record(finalize_payload, state, signer)?
            }
            Action::CreateRecordType(record_type_payload) => {
                self._create_record_type(record_type_payload, state, signer)?
            }
            Action::UpdateProperties(update_properties_payload) => self._update_properties(
                update_properties_payload,
                state,
                signer,
                payload.get_timestamp(),
            )?,
            Action::CreateProposal(proposal_payload) => {
                self._create_proposal(proposal_payload, state, signer, payload.get_timestamp())?
            }
            Action::AnswerProposal(answer_proposal_payload) => self._answer_proposal(
                answer_proposal_payload,
                state,
                signer,
                payload.get_timestamp(),
            )?,
            Action::RevokeReporter(revoke_reporter_payload) => {
                self._revoke_reporter(revoke_reporter_payload, state, signer)?
            }
        }
        Ok(())
    }
}

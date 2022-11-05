use bytes::BufMut;

use super::{mqtt_traits::{MqttPacketRead, MqttRead, MqttWrite, WireLength, MqttPacketWrite}, reason_codes::{SubAckReasonCode}, read_variable_integer, errors::{DeserializeError, SerializeError}, PropertyType, PacketType, write_variable_integer, variable_integer_len};

///3.9 SUBACK – Subscribe acknowledgement
/// A SUBACK packet is sent by the Server to the Client to confirm receipt and processing of a SUBSCRIBE packet.
/// A SUBACK packet contains a list of Reason Codes, that specify the maximum QoS level that was granted or the error which was found for each Subscription that was requested by the SUBSCRIBE.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct SubAck{
    pub packet_identifier: u16,
    pub properties: SubAckProperties,
    pub reason_codes: Vec<SubAckReasonCode>,
}


impl MqttPacketRead for SubAck{
    fn read(_: u8, _: usize,  mut buf: bytes::Bytes) -> Result<Self, super::errors::DeserializeError> {
        let packet_identifier = u16::read(&mut buf)?;
        let properties = SubAckProperties::read(&mut buf)?;
        let mut reason_codes = vec![];
        loop{

            let reason_code = SubAckReasonCode::read(&mut buf)?;

            reason_codes.push(reason_code);

            if buf.len() == 0{
                break;
            }
        }

        Ok(Self{
            packet_identifier,
            properties,
            reason_codes,
        })
    }
}

impl MqttPacketWrite for SubAck{
    fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), SerializeError> {
        buf.put_u16(self.packet_identifier);

        self.properties.write(buf)?;
        for reason_code in &self.reason_codes{
            reason_code.write(buf)?;
        }

        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct SubAckProperties{
    /// 3.8.2.1.2 Subscription Identifier
    /// 11 (0x0B) Byte, Identifier of the Subscription Identifier. 
    pub subscription_id: Option<usize>,

    /// 3.8.2.1.3 User Property
    /// 38 (0x26) Byte, Identifier of the User Property. 
    pub user_properties: Vec<(String, String)>,
}

impl MqttRead for SubAckProperties{
    fn read(buf: &mut bytes::Bytes) -> Result<Self, DeserializeError> {
        let (len, _) = read_variable_integer(buf)?;
        
        let mut properties = SubAckProperties::default();

        if len == 0{
            return Ok(properties);
        }
        else if buf.len() < len{
            return Err(DeserializeError::InsufficientData(buf.len(), len));
        }

        let mut properties_data = buf.split_to(len);

        loop{
            match PropertyType::read(&mut properties_data)? {
                PropertyType::SubscriptionIdentifier => {
                    if properties.subscription_id.is_none(){
                        let (subscription_id, _) = read_variable_integer(&mut properties_data)?;
    
                        properties.subscription_id = Some(subscription_id);
                    }
                    else{
                        return Err(DeserializeError::DuplicateProperty(PropertyType::SubscriptionIdentifier))
                    }
                },
                PropertyType::UserProperty => {                 
                    properties.user_properties.push((String::read(&mut properties_data)?, String::read(&mut properties_data)?));
                },
                e => return Err(DeserializeError::UnexpectedProperty(e, PacketType::Connect)),
            }

            if buf.len() == 0{
                break;
            }
        }
        Ok(properties)
    }
}

impl MqttWrite for SubAckProperties{
    fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), super::errors::SerializeError> {
        write_variable_integer(buf, self.wire_len())?;
        if let Some(sub_id) = self.subscription_id{
            PropertyType::SubscriptionIdentifier.write(buf)?;
            write_variable_integer(buf, sub_id)?;
        }
        for (key, value) in &self.user_properties{
            PropertyType::UserProperty.write(buf)?;
            key.write(buf)?;
            value.write(buf)?;
        }
        Ok(())
    }
}

impl WireLength for SubAckProperties{
    fn wire_len(&self) -> usize {
        let mut len = 0;
        if let Some(sub_id) = self.subscription_id{
            len += 1 + variable_integer_len(sub_id);
        }
        for (key, value) in &self.user_properties{
            len += 1 + key.wire_len() + value.wire_len();
        }
        len
    }
}


#[cfg(test)]
mod test {
    use bytes::BytesMut;
    use pretty_assertions::assert_eq;

    use super::SubAck;
    use crate::packets::mqtt_traits::{MqttPacketRead, MqttPacketWrite};

    #[test]
    fn read_write_suback() {
        let buf = vec![
            0x00, 
            0x0F, // variable header. pkid = 15
            0x00, // Property length 0
            0x01, // Payload reason code codes Granted QoS 1,
            0x80, // Payload Unspecified error
        ];

        let data = BytesMut::from(&buf[..]);
        let sub_ack = SubAck::read(0, 0, data.clone().into()).unwrap();

        let mut result = BytesMut::new(); 
        sub_ack.write(&mut result).unwrap();

        assert_eq!(
            data.to_vec(),
            result.to_vec()
        );
    }
}
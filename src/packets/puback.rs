use bytes::BufMut;

use super::{
    error::DeserializeError,
    mqtt_traits::{MqttRead, MqttWrite, VariableHeaderRead, VariableHeaderWrite, WireLength},
    read_variable_integer,
    reason_codes::PubAckReasonCode,
    variable_integer_len, write_variable_integer, PacketType, PropertyType,
};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct PubAck {
    pub packet_identifier: u16,
    pub reason_code: PubAckReasonCode,
    pub properties: PubAckProperties,
}

impl VariableHeaderRead for PubAck {
    fn read(
        _: u8,
        remaining_length: usize,
        mut buf: bytes::Bytes,
    ) -> Result<Self, DeserializeError> {
        // reason code and properties are optional if reasoncode is success and properties empty.
        if remaining_length == 2 {
            return Ok(Self {
                packet_identifier: u16::read(&mut buf)?,
                reason_code: PubAckReasonCode::Success,
                properties: PubAckProperties::default(),
            });
        }
        // Requires u16, u8 and at leasy 1 byte of variable integer prop length so at least 4 bytes
        else if remaining_length < 4 {
            return Err(DeserializeError::InsufficientData(
                "PubAck".to_string(),
                buf.len(),
                4,
            ));
        }

        let packet_identifier = u16::read(&mut buf)?;
        let reason_code = PubAckReasonCode::read(&mut buf)?;
        let properties = PubAckProperties::read(&mut buf)?;

        Ok(Self {
            packet_identifier,
            reason_code,
            properties,
        })
    }
}

impl VariableHeaderWrite for PubAck {
    fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), super::error::SerializeError> {
        buf.put_u16(self.packet_identifier);

        if self.reason_code == PubAckReasonCode::Success
            && self.properties.reason_string.is_none()
            && self.properties.user_properties.is_empty()
        {
            // nothing here
        } else if self.properties.reason_string.is_none()
            && self.properties.user_properties.is_empty()
        {
            self.reason_code.write(buf)?;
        } else {
            self.reason_code.write(buf)?;
            self.properties.write(buf)?;
        }
        Ok(())
    }
}

impl WireLength for PubAck {
    fn wire_len(&self) -> usize {
        if self.reason_code == PubAckReasonCode::Success
            && self.properties.reason_string.is_none()
            && self.properties.user_properties.is_empty()
        {
            // Only pkid
            2
        } else if self.properties.reason_string.is_none()
            && self.properties.user_properties.is_empty()
        {
            // pkid and reason code
            3
        } else {
            let prop_len = self.properties.wire_len();
            // pkid, reason code, length of the length of properties and lenght of properties
            3 + variable_integer_len(prop_len) + prop_len
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Default)]
pub struct PubAckProperties {
    pub reason_string: Option<String>,
    pub user_properties: Vec<(String, String)>,
}

impl PubAckProperties {
    pub fn is_empty(&self) -> bool {
        self.reason_string.is_none() && self.user_properties.is_empty()
    }
}

impl MqttRead for PubAckProperties {
    fn read(buf: &mut bytes::Bytes) -> Result<Self, super::error::DeserializeError> {
        let (len, _) = read_variable_integer(buf)?;

        if len == 0 {
            return Ok(Self::default());
        }
        if buf.len() < len {
            return Err(DeserializeError::InsufficientData(
                "PubAckProperties".to_string(),
                buf.len(),
                len,
            ));
        }

        let mut properties = PubAckProperties::default();

        loop {
            match PropertyType::from_u8(u8::read(buf)?)? {
                PropertyType::ReasonString => {
                    if properties.reason_string.is_some() {
                        return Err(DeserializeError::DuplicateProperty(
                            PropertyType::ReasonString,
                        ));
                    }
                    properties.reason_string = Some(String::read(buf)?);
                }
                PropertyType::UserProperty => properties
                    .user_properties
                    .push((String::read(buf)?, String::read(buf)?)),
                e => return Err(DeserializeError::UnexpectedProperty(e, PacketType::PubAck)),
            }
            if buf.is_empty() {
                break;
            }
        }
        Ok(properties)
    }
}

impl MqttWrite for PubAckProperties {
    fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), super::error::SerializeError> {
        let len = self.wire_len();

        write_variable_integer(buf, len)?;

        if let Some(reason_string) = &self.reason_string {
            PropertyType::ReasonString.write(buf)?;
            reason_string.write(buf)?;
        }
        for (key, value) in &self.user_properties {
            PropertyType::UserProperty.write(buf)?;
            key.write(buf)?;
            value.write(buf)?
        }

        Ok(())
    }
}

impl WireLength for PubAckProperties {
    fn wire_len(&self) -> usize {
        let mut len = 0;
        if let Some(reason_string) = &self.reason_string {
            len += reason_string.wire_len() + 1;
        }
        for (key, value) in &self.user_properties {
            len += 1 + key.wire_len() + value.wire_len();
        }

        len
    }
}

#[cfg(test)]
mod tests {
    use crate::packets::{
        mqtt_traits::{MqttRead, MqttWrite, VariableHeaderRead, VariableHeaderWrite, WireLength},
        puback::{PubAck, PubAckProperties},
        reason_codes::PubAckReasonCode,
        write_variable_integer, PropertyType,
    };
    use bytes::{BufMut, Bytes, BytesMut};

    #[test]
    fn test_wire_len() {
        let mut puback = PubAck {
            packet_identifier: 12,
            reason_code: PubAckReasonCode::Success,
            properties: PubAckProperties::default(),
        };

        let mut buf = BytesMut::new();

        puback.write(&mut buf).unwrap();

        assert_eq!(2, puback.wire_len());
        assert_eq!(2, buf.len());

        puback.reason_code = PubAckReasonCode::NotAuthorized;
        buf.clear();
        puback.write(&mut buf).unwrap();

        assert_eq!(3, puback.wire_len());
        assert_eq!(3, buf.len());
    }

    #[test]
    fn test_read_simple_puback() {
        let stream = &[
            0x00, 0x0C, // Packet identifier = 12
            0x00, // Reason code success
            0x00, // no properties
        ];
        let buf = Bytes::from(&stream[..]);
        let p_ack = PubAck::read(0, 4, buf).unwrap();

        let expected = PubAck {
            packet_identifier: 12,
            reason_code: PubAckReasonCode::Success,
            properties: PubAckProperties::default(),
        };

        assert_eq!(expected, p_ack);
    }

    #[test]
    fn test_read_write_puback_with_properties() {
        let mut buf = BytesMut::new();

        buf.put_u16(65_535u16);
        buf.put_u8(0x99);

        let mut properties = BytesMut::new();
        PropertyType::ReasonString.write(&mut properties).unwrap();
        "reason string, test 1-2-3.".write(&mut properties).unwrap();
        PropertyType::UserProperty.write(&mut properties).unwrap();
        "This is the key".write(&mut properties).unwrap();
        "This is the value".write(&mut properties).unwrap();
        PropertyType::UserProperty.write(&mut properties).unwrap();
        "Another thingy".write(&mut properties).unwrap();
        "The thingy".write(&mut properties).unwrap();

        write_variable_integer(&mut buf, properties.len()).unwrap();

        buf.extend(properties);

        // flags can be 0 because not used.
        // remaining_length must be at least 4
        let p_ack = PubAck::read(0, buf.len(), buf.clone().into()).unwrap();

        let mut result = BytesMut::new();
        p_ack.write(&mut result).unwrap();

        assert_eq!(buf.to_vec(), result.to_vec());
    }

    #[test]
    fn test_properties() {
        let mut properties_data = BytesMut::new();
        PropertyType::ReasonString
            .write(&mut properties_data)
            .unwrap();
        "reason string, test 1-2-3."
            .write(&mut properties_data)
            .unwrap();
        PropertyType::UserProperty
            .write(&mut properties_data)
            .unwrap();
        "This is the key".write(&mut properties_data).unwrap();
        "This is the value".write(&mut properties_data).unwrap();
        PropertyType::UserProperty
            .write(&mut properties_data)
            .unwrap();
        "Another thingy".write(&mut properties_data).unwrap();
        "The thingy".write(&mut properties_data).unwrap();

        let mut buf = BytesMut::new();
        write_variable_integer(&mut buf, properties_data.len()).unwrap();
        buf.extend(properties_data);

        let properties = PubAckProperties::read(&mut buf.clone().into()).unwrap();
        let mut result = BytesMut::new();
        properties.write(&mut result).unwrap();

        assert_eq!(buf.to_vec(), result.to_vec());
    }

    #[test]
    fn no_reason_code_or_props() {
        let mut buf = BytesMut::new();

        buf.put_u16(65_535u16);
        let p_ack = PubAck::read(0, buf.len(), buf.clone().into()).unwrap();

        let mut result = BytesMut::new();
        p_ack.write(&mut result).unwrap();

        let expected = PubAck {
            packet_identifier: 65535,
            reason_code: PubAckReasonCode::Success,
            properties: PubAckProperties::default(),
        };
        let mut result = BytesMut::new();
        expected.write(&mut result).unwrap();

        assert_eq!(expected, p_ack);
        assert_eq!(buf.to_vec(), result.to_vec());
    }
}

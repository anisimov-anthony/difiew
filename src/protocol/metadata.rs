use bincode::{Decode, Encode};
use libp2p::PeerId;

#[derive(Decode, Encode, Debug, Clone, PartialEq)]
pub struct MetaData {
    pub peer_id_str: String,
    pub local_time: u128,
}

impl MetaData {
    pub fn new(peer_id: PeerId, local_time: u128) -> Self {
        Self {
            peer_id_str: peer_id.to_string(),
            local_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::PeerId;
    use std::str::FromStr;

    #[test]
    fn test_new_creates_correct_metadata() {
        let peer_id =
            PeerId::from_str("12D3KooWHuHkD4ySUmGACxMcd7xUsZo92C5wpadjrFNHb8AYQvX1").unwrap();
        let local_time = 1234567890123456789u128;

        let meta = MetaData::new(peer_id, local_time);

        assert_eq!(meta.peer_id_str, peer_id.to_string());
        assert_eq!(meta.local_time, local_time);
    }

    #[test]
    fn test_bincode_serialization_roundtrip() {
        let peer_id =
            PeerId::from_str("12D3KooWHuHkD4ySUmGACxMcd7xUsZo92C5wpadjrFNHb8AYQvX1").unwrap();
        let local_time = 9876543210987654321u128;
        let original = MetaData::new(peer_id, local_time);

        let encoded: Vec<u8> =
            bincode::encode_to_vec(&original, bincode::config::standard()).unwrap();

        let (decoded, _): (MetaData, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

        assert_eq!(decoded.peer_id_str, original.peer_id_str);
        assert_eq!(decoded.local_time, original.local_time);
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_metadata_clone_and_debug() {
        let peer_id =
            PeerId::from_str("12D3KooWHuHkD4ySUmGACxMcd7xUsZo92C5wpadjrFNHb8AYQvX1").unwrap();
        let local_time = 5555555555555555555u128;
        let meta = MetaData::new(peer_id, local_time);

        let cloned = meta.clone();
        assert_eq!(meta.peer_id_str, cloned.peer_id_str);
        assert_eq!(meta.local_time, cloned.local_time);

        let debug_str = format!("{:?}", meta);
        assert!(debug_str.contains(&meta.peer_id_str));
        assert!(debug_str.contains(&meta.local_time.to_string()));
    }

    #[test]
    fn test_different_peer_ids_produce_different_strings() {
        let peer1 =
            PeerId::from_str("12D3KooWHuHkD4ySUmGACxMcd7xUsZo92C5wpadjrFNHb8AYQvX1").unwrap();
        let peer2 =
            PeerId::from_str("12D3KooWRj6mv1m6K4LSjCRP7ghtHFVh2JRW6Sq9j4nMgUCJui1v").unwrap();

        let meta1 = MetaData::new(peer1, 100);
        let meta2 = MetaData::new(peer2, 100);

        assert_ne!(meta1.peer_id_str, meta2.peer_id_str);
    }

    #[test]
    fn test_zero_time_is_handled_correctly() {
        let peer_id =
            PeerId::from_str("12D3KooWHuHkD4ySUmGACxMcd7xUsZo92C5wpadjrFNHb8AYQvX1").unwrap();
        let meta = MetaData::new(peer_id, 0);

        assert_eq!(meta.local_time, 0);
        let encoded: Vec<u8> = bincode::encode_to_vec(&meta, bincode::config::standard()).unwrap();
        let (decoded, _): (MetaData, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(decoded.local_time, 0);
    }

    #[test]
    fn test_max_u128_time_is_handled() {
        let peer_id =
            PeerId::from_str("12D3KooWHuHkD4ySUmGACxMcd7xUsZo92C5wpadjrFNHb8AYQvX1").unwrap();
        let max_time = u128::MAX;
        let meta = MetaData::new(peer_id, max_time);

        assert_eq!(meta.local_time, max_time);
        let encoded: Vec<u8> = bincode::encode_to_vec(&meta, bincode::config::standard()).unwrap();
        let (decoded, _): (MetaData, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(decoded.local_time, max_time);
    }
}

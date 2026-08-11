use bedrock_codec::prelude::*;

use crate::direction::Direction;
use crate::mapping::IdShift;
use crate::packet_ids::ids;
use crate::rewriter::SoundRewriter;
use crate::translator::Translator;

const SOUND: IdShift = IdShift::inserted(2, 599);
const HEARTBEAT_KEY: u32 = 126;

const V975_ONLY: &[u16] = &[ids::SERVER_STORE_INFO, ids::SERVER_PRESENCE_INFO];

const DEBUG_DRAWER: &[u16] = &[ids::SERVER_SCRIPT_DEBUG_DRAWER];

const TELEMETRY: &[u16] = &[ids::SERVERBOUND_DIAGNOSTICS];

fn air() -> Item {
    Item {
        network_id: 0,
        count: 0,
        aux_value: 0,
        has_net_id: false,
        stack_net_id: 0,
        net_id_variant: 0,
        block_runtime_id: 0,
        user_data: Vec::new(),
    }
}

fn convert_item(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    if to_v975 {
        w.map::<ItemInstance, ItemInstanceV975>()?;
    } else {
        w.map::<ItemInstanceV975, ItemInstance>()?;
    }
    Ok(())
}

fn byte_width(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    if to_v975 {
        let v = w.read::<Byte>()?;
        w.write::<UVarInt>(u32::from(v));
    } else {
        let v = w.read::<UVarInt>()?;
        w.write::<Byte>(v.min(0xFF) as u8);
    }
    Ok(())
}

fn passthrough_optional<C: Codec>(w: &mut PacketWrapper) -> Result<bool> {
    let present = w.passthrough::<Bool>()?;
    if present {
        w.passthrough::<C>()?;
    }
    Ok(present)
}

fn start_game(w: &mut PacketWrapper) -> Result<()> {
    w.passthrough::<VarInt64>()?;
    w.passthrough::<UVarInt64>()?;
    w.passthrough::<VarInt>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec2>()?;

    w.passthrough::<LevelSettingsV944>()?;

    w.passthrough::<Str>()?;
    w.passthrough::<Str>()?;
    w.passthrough::<Str>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<VarInt>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<Int64Le>()?;
    w.passthrough::<VarInt>()?;

    w.passthrough_each(|w| {
        w.passthrough::<Str>()?;
        w.passthrough::<NamedCompoundTag>()?;
        Ok(())
    })?;

    w.passthrough::<Str>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<Str>()?;
    w.passthrough::<NamedCompoundTag>()?;

    w.read::<Int64Le>()?;
    w.write::<Int64Le>(0);

    Ok(())
}

fn player_equipment(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    convert_item(w, to_v975)?;
    byte_width(w, to_v975)?;
    byte_width(w, to_v975)?;
    byte_width(w, to_v975)?;
    Ok(())
}

fn inventory_slot(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    w.passthrough::<UVarInt>()?;
    w.passthrough::<UVarInt>()?;

    if to_v975 {
        let name = w.read::<Byte>()?;
        let has_dynamic = w.read::<Bool>()?;
        let dynamic = if has_dynamic { w.read::<UIntLe>()? } else { 0 };
        w.write::<Bool>(true);
        w.write::<Byte>(name);
        w.write::<Bool>(has_dynamic);
        if has_dynamic {
            w.write::<UIntLe>(dynamic);
        }

        let storage = w.read::<ItemInstance>()?;
        if storage.is_air() {
            w.write::<Bool>(false);
        } else {
            w.write::<Bool>(true);
            w.write::<ItemInstanceV975>(storage);
        }
    } else {
        if w.read::<Bool>()? {
            let name = w.read::<Byte>()?;
            let has_dynamic = w.read::<Bool>()?;
            let dynamic = if has_dynamic { w.read::<UIntLe>()? } else { 0 };
            w.write::<Byte>(name);
            w.write::<Bool>(has_dynamic);
            if has_dynamic {
                w.write::<UIntLe>(dynamic);
            }
        } else {
            w.write::<Byte>(0);
            w.write::<Bool>(false);
        }

        if w.read::<Bool>()? {
            let storage = w.read::<ItemInstanceV975>()?;
            w.write::<ItemInstance>(storage);
        } else {
            w.write::<ItemInstance>(air());
        }
    }

    convert_item(w, to_v975)?;
    Ok(())
}

fn player_enchant_options(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    w.passthrough_each(|w| {
        if to_v975 {
            let cost = w.read::<UVarInt>()?;
            w.write::<Byte>(cost.min(0xFF) as u8);
        } else {
            let cost = w.read::<Byte>()?;
            w.write::<UVarInt>(u32::from(cost));
        }
        w.passthrough::<IntLe>()?;
        for _ in 0..3 {
            w.passthrough_each(|w| {
                w.passthrough::<Byte>()?;
                w.passthrough::<Byte>()?;
                Ok(())
            })?;
        }
        w.passthrough::<Str>()?;
        w.passthrough::<UVarInt>()?;
        Ok(())
    })?;
    Ok(())
}

fn locator_bar(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    w.passthrough_each(|w| {
        w.passthrough::<Uuid>()?;
        w.passthrough::<UIntLe>()?;
        passthrough_optional::<Bool>(w)?;
        if w.passthrough::<Bool>()? {
            w.passthrough::<Vec3>()?;
            w.passthrough::<VarInt>()?;
        }

        if to_v975 {
            if w.read::<Bool>()? {
                w.read::<UIntLe>()?;
            }
            w.write::<Bool>(false);
            w.write::<Bool>(false);
        } else {
            if w.read::<Bool>()? {
                w.read::<Str>()?;
            }
            if w.read::<Bool>()? {
                w.read::<Vec2>()?;
            }
            w.write::<Bool>(false);
        }

        passthrough_optional::<IntLe>(w)?;
        passthrough_optional::<Bool>(w)?;
        passthrough_optional::<VarInt64>(w)?;
        w.passthrough::<UVarInt>()?;
        Ok(())
    })?;
    Ok(())
}

fn play_sound(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    w.passthrough::<Str>()?;
    w.passthrough::<BlockPos>()?;
    w.passthrough::<FloatLe>()?;
    w.passthrough::<FloatLe>()?;
    if to_v975 {
        w.write::<Bool>(false);
    } else if w.read::<Bool>()? {
        w.read::<UInt64Le>()?;
    }
    Ok(())
}

fn actor_event(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    w.passthrough::<Byte>()?;
    w.passthrough::<VarInt>()?;
    if to_v975 {
        w.write::<Bool>(false);
    } else if w.read::<Bool>()? {
        w.read::<Vec3>()?;
    }
    Ok(())
}

fn level_sound_event(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    let event = w.read::<UVarInt>()?;
    let mapped = if to_v975 { SOUND.up(event) } else { SOUND.down(event) };
    w.write::<UVarInt>(mapped);

    w.passthrough::<Vec3>()?;
    w.passthrough::<VarInt>()?;
    w.passthrough::<Str>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<Int64Le>()?;

    if to_v975 {
        w.write::<Bool>(false);
    } else if w.read::<Bool>()? {
        w.read::<Vec3>()?;
    }
    Ok(())
}

fn party_changed(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    if w.passthrough::<Bool>()? {
        w.passthrough::<Str>()?;
        if to_v975 {
            w.write::<Bool>(false);
        } else {
            w.read::<Bool>()?;
        }
    }
    Ok(())
}

fn update_client_options(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    passthrough_optional::<Byte>(w)?;
    if to_v975 {
        w.write::<Bool>(false);
    } else if w.read::<Bool>()? {
        w.read::<Bool>()?;
    }
    Ok(())
}

fn passthrough_bitset(w: &mut PacketWrapper) -> Result<()> {
    loop {
        if (w.passthrough::<Byte>()? & 0x80) == 0 {
            return Ok(());
        }
    }
}

fn client_movement_prediction_sync(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    passthrough_bitset(w)?;
    for _ in 0..9 {
        w.passthrough::<FloatLe>()?;
    }
    if to_v975 {
        for _ in 0..3 {
            w.write::<FloatLe>(0.0);
        }
    } else {
        for _ in 0..3 {
            w.read::<FloatLe>()?;
        }
    }
    w.passthrough::<VarInt64>()?;
    w.passthrough::<Bool>()?;
    Ok(())
}

fn make(name: &'static str, server: u32, client: u32, to_v975_clientbound: bool) -> Translator {
    let to_client = to_v975_clientbound;
    let to_server = !to_v975_clientbound;

    let mut step = Translator::new(name, server, client)
        .clientbound(ids::START_GAME, move |w, _| start_game(w))
        .clientbound(ids::PLAYER_EQUIPMENT, move |w, _| {
            player_equipment(w, to_client)
        })
        .clientbound(ids::INVENTORY_SLOT, move |w, _| {
            inventory_slot(w, to_client)
        })
        .clientbound(ids::PLAYER_ENCHANT_OPTIONS, move |w, _| {
            player_enchant_options(w, to_client)
        })
        .clientbound(ids::LOCATOR_BAR, move |w, _| locator_bar(w, to_client))
        .clientbound(ids::PLAY_SOUND, move |w, _| play_sound(w, to_client))
        .clientbound(ids::ACTOR_EVENT, move |w, _| actor_event(w, to_client))
        .clientbound(ids::LEVEL_SOUND_EVENT, move |w, _| {
            level_sound_event(w, to_client)
        })
        .serverbound(ids::LEVEL_SOUND_EVENT, move |w, _| {
            level_sound_event(w, to_server)
        })
        .serverbound(ids::PLAYER_EQUIPMENT, move |w, _| {
            player_equipment(w, to_server)
        })
        .serverbound(ids::ACTOR_EVENT, move |w, _| actor_event(w, to_server))
        .serverbound(ids::PARTY_CHANGED, move |w, _| party_changed(w, to_server))
        .serverbound(ids::UPDATE_CLIENT_OPTIONS, move |w, _| {
            update_client_options(w, to_server)
        })
        .serverbound(ids::CLIENT_MOVEMENT_PREDICTION_SYNC, move |w, _| {
            client_movement_prediction_sync(w, to_server)
        })
        .cancel_all(Direction::Clientbound, DEBUG_DRAWER)
        .cancel_all(Direction::Serverbound, TELEMETRY);

    if !to_v975_clientbound {
        step = step.cancel_all(Direction::Clientbound, V975_ONLY);
    } else {
        step = step.cancel_all(Direction::Serverbound, V975_ONLY);
    }

    let client_is_newer = client > server;
    SoundRewriter::new(SOUND, client_is_newer, HEARTBEAT_KEY)
        .with_item_encoding_change()
        .without_level_sound_event()
        .register(step)
}

pub fn downgrade() -> Translator {
    make("v975->v944", 944, 975, true)
}

pub fn upgrade() -> Translator {
    make("v944->v975", 975, 944, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_item() -> Item {
        Item {
            network_id: 42,
            count: 7,
            aux_value: 3,
            has_net_id: true,
            stack_net_id: 11,
            net_id_variant: 0,
            block_runtime_id: 5,
            user_data: vec![9, 9],
        }
    }

    fn round_trip(v944: &[u8], f: fn(&mut PacketWrapper, bool) -> Result<()>) {
        let mut up = PacketWrapper::new(v944);
        f(&mut up, true).unwrap();
        let v975 = up.finish();

        let mut down = PacketWrapper::new(&v975);
        f(&mut down, false).unwrap();
        assert_eq!(down.finish(), v944, "round trip changed the packet");
    }

    #[test]
    fn item_survives_a_round_trip_through_v975() {
        let mut w = Writer::new();
        ItemInstance::write(&mut w, &sample_item());
        let v944 = w.into_vec();

        let mut up = PacketWrapper::new(&v944);
        convert_item(&mut up, true).unwrap();
        let v975 = up.finish();

        let mut down = PacketWrapper::new(&v975);
        convert_item(&mut down, false).unwrap();
        assert_eq!(down.finish(), v944);
    }

    #[test]
    fn air_survives_despite_the_shortcut_difference() {
        let v944 = vec![0x00u8];
        let mut up = PacketWrapper::new(&v944);
        convert_item(&mut up, true).unwrap();
        let v975 = up.finish();
        assert!(v975.len() > 1);

        let mut down = PacketWrapper::new(&v975);
        convert_item(&mut down, false).unwrap();
        assert_eq!(down.finish(), v944);
    }

    #[test]
    fn player_equipment_widens_and_narrows_the_slot_fields() {
        let mut w = Writer::new();
        UVarInt64::write(&mut w, &7);
        ItemInstance::write(&mut w, &sample_item());
        Byte::write(&mut w, &2);
        Byte::write(&mut w, &0);
        Byte::write(&mut w, &1);
        let v944 = w.into_vec();

        let mut up = PacketWrapper::new(&v944);
        player_equipment(&mut up, true).unwrap();
        let v975 = up.finish();
        assert!(v975.len() > v944.len(), "the item form should have grown");

        let mut down = PacketWrapper::new(&v975);
        player_equipment(&mut down, false).unwrap();
        assert_eq!(down.finish(), v944);
    }

    #[test]
    fn actor_event_round_trips() {
        let mut w = Writer::new();
        UVarInt64::write(&mut w, &9);
        Byte::write(&mut w, &3);
        VarInt::write(&mut w, &-4);
        round_trip(&w.into_vec(), actor_event);
    }

    #[test]
    fn party_changed_round_trips_with_and_without_the_optional() {
        let mut absent = Writer::new();
        Bool::write(&mut absent, &false);
        round_trip(&absent.into_vec(), party_changed);

        let mut present = Writer::new();
        Bool::write(&mut present, &true);
        Str::write(&mut present, &"abc".to_string());
        round_trip(&present.into_vec(), party_changed);
    }

    #[test]
    fn update_client_options_round_trips() {
        let mut none = Writer::new();
        Bool::write(&mut none, &false);
        round_trip(&none.into_vec(), update_client_options);

        let mut some = Writer::new();
        Bool::write(&mut some, &true);
        Byte::write(&mut some, &2);
        round_trip(&some.into_vec(), update_client_options);
    }

    #[test]
    fn movement_prediction_sync_moves_exactly_three_floats() {
        let mut w = Writer::new();
        Byte::write(&mut w, &0x00);
        for i in 0..9 {
            FloatLe::write(&mut w, &(i as f32));
        }
        VarInt64::write(&mut w, &123);
        Bool::write(&mut w, &true);
        let v944 = w.into_vec();

        let mut up = PacketWrapper::new(&v944);
        client_movement_prediction_sync(&mut up, true).unwrap();
        let v975 = up.finish();
        assert_eq!(v975.len(), v944.len() + 12);

        let mut down = PacketWrapper::new(&v975);
        client_movement_prediction_sync(&mut down, false).unwrap();
        assert_eq!(down.finish(), v944);
    }

    #[test]
    fn inventory_slot_round_trips_with_a_real_storage_item() {
        let mut w = Writer::new();
        UVarInt::write(&mut w, &1);
        UVarInt::write(&mut w, &4);
        Byte::write(&mut w, &12);
        Bool::write(&mut w, &false);
        ItemInstance::write(&mut w, &sample_item());
        ItemInstance::write(&mut w, &sample_item());
        round_trip(&w.into_vec(), inventory_slot);
    }

    #[test]
    fn inventory_slot_maps_air_storage_to_an_absent_optional() {
        let mut w = Writer::new();
        UVarInt::write(&mut w, &0);
        UVarInt::write(&mut w, &0);
        Byte::write(&mut w, &0);
        Bool::write(&mut w, &false);
        ItemInstance::write(&mut w, &air());
        ItemInstance::write(&mut w, &sample_item());
        round_trip(&w.into_vec(), inventory_slot);
    }

    #[test]
    fn player_enchant_options_round_trips() {
        let mut w = Writer::new();
        UVarInt::write(&mut w, &1);
        UVarInt::write(&mut w, &30);
        IntLe::write(&mut w, &2);
        for _ in 0..3 {
            UVarInt::write(&mut w, &1);
            Byte::write(&mut w, &5);
            Byte::write(&mut w, &2);
        }
        Str::write(&mut w, &"sharp".to_string());
        UVarInt::write(&mut w, &77);
        round_trip(&w.into_vec(), player_enchant_options);
    }

    #[test]
    fn level_sound_event_shifts_the_id_back_and_forth() {
        let mut w = Writer::new();
        UVarInt::write(&mut w, &700);
        Vec3::write(&mut w, &(1.0, 2.0, 3.0));
        VarInt::write(&mut w, &0);
        Str::write(&mut w, &"pig".to_string());
        Bool::write(&mut w, &false);
        Bool::write(&mut w, &false);
        Int64Le::write(&mut w, &-1);
        round_trip(&w.into_vec(), level_sound_event);
    }

    #[test]
    fn v975_only_packets_are_dropped_toward_v944() {
        let up = upgrade();
        for id in V975_ONLY {
            assert!(
                up.is_cancelled(Direction::Clientbound, *id),
                "packet {id} should not reach a v944 client"
            );
        }
        let down = downgrade();
        for id in V975_ONLY {
            assert!(
                down.is_cancelled(Direction::Serverbound, *id),
                "packet {id} should not reach a v944 server"
            );
        }
    }

    #[test]
    fn debug_drawer_and_diagnostics_are_cancelled_in_both_steps() {
        for step in [downgrade(), upgrade()] {
            assert!(step.is_cancelled(Direction::Clientbound, ids::SERVER_SCRIPT_DEBUG_DRAWER));
            assert!(step.is_cancelled(Direction::Serverbound, ids::SERVERBOUND_DIAGNOSTICS));
        }
    }

    #[test]
    fn step_endpoints_are_the_documented_pair() {
        let d = downgrade();
        assert_eq!((d.server_protocol, d.client_protocol), (944, 975));
        let u = upgrade();
        assert_eq!((u.server_protocol, u.client_protocol), (975, 944));
    }
}

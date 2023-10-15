use std::sync::Arc;

use super::Client;
use super::Server;
use super::player::Player;
use super::player::ServerPlayer;


pub(super) fn login(server: &mut Server, name : String, client: Client) {
    if server.is_logged_in(&name){
        _ = client.try_send(Arc::new(*b"\x01\x11")); //Already logged in code
    } else {
        _ = client.try_send(Arc::new(*b"\x01\x00")); //Sucessfull log in
        //todo send this later if some things can go wrong
        let mut server_player = match server.is_known(&name) {
            Some(player) => {
                ServerPlayer{player, package_writer: client, player_id: 0}
            }
            None => {
                let player = Player::new(name);
                server.players.push(player.clone());
                ServerPlayer{player, package_writer: client, player_id: 0}
            }
        };

        if let Some(slot) = crate::misc::first_none(&server.connected_players) {
            server_player.player_id = slot;
            server.connected_players[slot] = Some(server_player);
        } else {
            server_player.player_id = server.connected_players.len();
            server.connected_players.push(Some(server_player));
        }
    }
}
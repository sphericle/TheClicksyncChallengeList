use super::{Creator, Creators};
use crate::{
    citext::CiStr,
    error::PointercrateError,
    model::{user::User, Demon},
    operation::Get,
    permissions::{self, AccessRestrictions},
    schema::creators,
    Result,
};
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};

impl<'a> Get<&'a CiStr> for Creators {
    fn get(name: &'a CiStr, connection: &PgConnection) -> Result<Self> {
        super::creators_of(name)
            .load(connection)
            .map(Creators)
            .map_err(PointercrateError::database)
    }
}

impl Get<(i16, i32)> for Creator {
    fn get((demon_position, player_id): (i16, i32), connection: &PgConnection) -> Result<Self> {
        let demon = Demon::get(demon_position, connection)?;

        creators::table
            .select((creators::demon, creators::creator))
            .filter(creators::demon.eq(&demon.name))
            .filter(creators::creator.eq(&player_id))
            .get_result(connection)
            .map_err(PointercrateError::database)
    }
}

impl AccessRestrictions for Creator {
    fn pre_delete(&self, user: Option<&User>) -> Result<()> {
        permissions::demand(perms!(ListModerator or ListAdministrator), user)
    }
}

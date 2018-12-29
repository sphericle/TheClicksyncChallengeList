use super::{All, Model};
use crate::{
    config::{EXTENDED_LIST_SIZE, LIST_SIZE},
    error::PointercrateError,
    model::player::Player,
    operation::Get,
    schema::{demon_publisher_verifier_join, demons, players},
    Result,
};
use diesel::{
    dsl::max, expression::bound::Bound, pg::Pg, query_builder::BoxedSelectStatement, sql_types,
    BoolExpressionMethods, Connection, Expression, ExpressionMethods, JoinOnDsl, PgConnection,
    QueryDsl, QueryResult, Queryable, RunQueryDsl,
};
use serde::{ser::SerializeMap, Serialize, Serializer};
use serde_derive::Serialize;
use std::fmt::Display;

mod get;
mod paginate;
mod patch;
mod post;

pub use self::{paginate::DemonPagination, patch::PatchDemon, post::PostDemon};

/// Struct modelling a demon in the database
#[derive(Debug, Identifiable, Serialize, Hash)]
#[table_name = "demons"]
#[primary_key("name")]
pub struct Demon {
    /// The [`Demon`]'s Geometry Dash level name
    pub name: String,

    /// The [`Demon`]'s position on the demonlist
    ///
    /// Positions for consecutive demons are always consecutive positive integers
    pub position: i16,

    /// The minimal progress a [`Player`] must achieve on this [`Demon`] to have their record
    /// accepted
    pub requirement: i16,

    pub video: Option<String>,

    // TODO: remove this field
    description: Option<String>,
    // TODO: remove this field
    notes: Option<String>,

    /// The player-ID of this [`Demon`]'s verifer
    pub verifier: Player,

    /// The player-ID of this [`Demon`]'s publisher
    pub publisher: Player,
}

/// Struct modelling a minimal representation of a [`Demon`] in the database
///
/// These representations are used whenever a different object references a demon, or when a list of
/// demons is requested
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct PartialDemon {
    pub name: String,
    pub position: i16,
    // TODO: when implemented return host here instead of publisher
    pub publisher: String,
}

impl Queryable<(sql_types::Text, sql_types::SmallInt, sql_types::Text), Pg> for PartialDemon {
    type Row = (String, i16, String);

    fn build(row: Self::Row) -> Self {
        PartialDemon {
            name: row.0,
            position: row.1,
            publisher: row.2,
        }
    }
}

impl Queryable<<AllColumns as Expression>::SqlType, Pg> for Demon {
    type Row = (
        String,
        i16,
        i16,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
        i32,
        bool,
        String,
        i32,
        bool,
    );

    fn build(row: Self::Row) -> Self {
        Demon {
            name: row.0,
            position: row.1,
            requirement: row.2,
            video: row.3,
            description: row.4,
            notes: row.5,
            verifier: Player {
                name: row.6,
                id: row.7,
                banned: row.8,
            },
            publisher: Player {
                name: row.9,
                id: row.10,
                banned: row.11,
            },
        }
    }
}

impl Serialize for PartialDemon {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(4))?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("position", &self.position)?;
        map.serialize_entry("publisher", &self.publisher)?;
        map.serialize_entry("state", &ListState::from(self.position).to_string())?;
        map.end()
    }
}

impl Model for PartialDemon {
    type From = diesel::query_source::joins::JoinOn<
        diesel::query_source::joins::Join<
            demons::table,
            players::table,
            diesel::query_source::joins::Inner,
        >,
        diesel::expression::operators::Eq<demons::columns::publisher, players::columns::id>,
    >;
    type Selection = (demons::name, demons::position, players::name);

    fn from() -> Self::From {
        diesel::query_source::joins::Join::new(
            demons::table,
            players::table,
            diesel::query_source::joins::Inner,
        )
        .on(demons::publisher.eq(players::id))
    }

    fn selection() -> Self::Selection {
        (demons::name, demons::position, players::name)
    }
}

impl Model for Demon {
    type From = diesel::query_source::joins::JoinOn<
        diesel::query_source::joins::Join<
            demons::table,
            demon_publisher_verifier_join::table,
            diesel::query_source::joins::Inner,
        >,
        diesel::dsl::And<
            diesel::expression::operators::Eq<
                demons::publisher,
                demon_publisher_verifier_join::pid,
            >,
            diesel::expression::operators::Eq<demons::verifier, demon_publisher_verifier_join::vid>,
        >,
    >;
    type Selection = AllColumns;

    fn from() -> Self::From {
        diesel::query_source::joins::Join::new(
            demons::table,
            demon_publisher_verifier_join::table,
            diesel::query_source::joins::Inner,
        )
        .on(demons::publisher
            .eq(demon_publisher_verifier_join::pid)
            .and(demons::verifier.eq(demon_publisher_verifier_join::vid)))
    }

    fn selection() -> Self::Selection {
        ALL_COLUMNS
    }
}

/// Enum encoding the 3 different parts of the demonlist
#[derive(Debug)]
pub enum ListState {
    /// The main part of the demonlist, ranging from position 1 onwards to [`LIST_SIZE`]
    /// (inclusive)
    Main,

    /// The extended part of the demonlist, ranging from [`LIST_SIZE`] (exclusive) onwards to
    /// [`EXTENDED_LIST_SIZE`] (inclusive)
    Extended,

    /// The legacy part of the demonlist, starting at [`EXTENDED_LIST_SIZE`] (exclusive) and being
    /// theoretically unbounded
    Legacy,
}

impl From<i16> for ListState {
    /// Calculates the [`ListState`] of [`Demon`] based on its [`Demon::position`]
    fn from(position: i16) -> ListState {
        if position <= *LIST_SIZE {
            ListState::Main
        } else if position <= *EXTENDED_LIST_SIZE {
            ListState::Extended
        } else {
            ListState::Legacy
        }
    }
}

impl Display for ListState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ListState::Main => write!(f, "MAIN"),
            ListState::Extended => write!(f, "EXTENDED"),
            ListState::Legacy => write!(f, "LEGACY"),
        }
    }
}

type AllColumns = (
    demons::name,
    demons::position,
    demons::requirement,
    demons::video,
    demons::description,
    demons::notes,
    demon_publisher_verifier_join::pname,
    demon_publisher_verifier_join::pid,
    demon_publisher_verifier_join::pbanned,
    demon_publisher_verifier_join::vname,
    demon_publisher_verifier_join::vid,
    demon_publisher_verifier_join::vbanned,
);

const ALL_COLUMNS: AllColumns = (
    demons::name,
    demons::position,
    demons::requirement,
    demons::video,
    demons::description,
    demons::notes,
    demon_publisher_verifier_join::pname,
    demon_publisher_verifier_join::pid,
    demon_publisher_verifier_join::pbanned,
    demon_publisher_verifier_join::vname,
    demon_publisher_verifier_join::vid,
    demon_publisher_verifier_join::vbanned,
);

type WithName<'a> = diesel::dsl::Eq<demons::name, Bound<sql_types::Text, &'a str>>;
type ByName<'a> = diesel::dsl::Filter<All<Demon>, WithName<'a>>;

type WithPosition = diesel::dsl::Eq<demons::position, Bound<sql_types::Int2, i16>>;
type ByPosition = diesel::dsl::Filter<All<Demon>, WithPosition>;

impl Demon {
    /// Constructs a diesel query returning all columns of demons whose name matches the given
    /// string
    pub fn by_name(name: &str) -> ByName {
        Demon::all().filter(demons::name.eq(name))
    }

    /// Constructs a diesel query returning all columns of position whose name matches the given i16
    pub fn by_position(position: i16) -> ByPosition {
        Demon::all().filter(demons::position.eq(position))
    }

    /// Increments the position of all demons with positions equal to or greater than the given one,
    /// by one.
    pub fn shift_down(starting_at: i16, connection: &PgConnection) -> QueryResult<()> {
        diesel::update(demons::table)
            .filter(demons::position.ge(starting_at))
            .set(demons::position.eq(demons::position + 1))
            .execute(connection)
            .map(|_| ())
    }

    /// Decrements the position of all demons with positions equal to or smaller than the given one,
    /// by one.
    pub fn shift_up(until: i16, connection: &PgConnection) -> QueryResult<()> {
        diesel::update(demons::table)
            .filter(demons::position.le(until))
            .set(demons::position.eq(demons::position - 1))
            .execute(connection)
            .map(|_| ())
    }

    pub fn mv(&self, to: i16, connection: &PgConnection) -> QueryResult<()> {
        if to > self.position {
            diesel::update(demons::table)
                .filter(demons::position.gt(self.position))
                .filter(demons::position.le(to))
                .set(demons::position.eq(demons::position - 1))
                .execute(connection)?;
        } else if to < self.position {
            diesel::update(demons::table)
                .filter(demons::position.ge(to))
                .filter(demons::position.gt(self.position))
                .set(demons::position.eq(demons::position + 1))
                .execute(connection)?;
        }

        if to != self.position {
            // alright, diesel::update(self) errors out for some reason
            diesel::update(demons::table)
                .filter(demons::name.eq(&self.name))
                .set(demons::position.eq(to))
                .execute(connection)?;
        }

        Ok(())
    }

    pub fn max_position(connection: &PgConnection) -> Result<i16> {
        let option = demons::table
            .select(max(demons::position))
            .get_result::<Option<i16>>(connection)?;

        Ok(option.unwrap_or(0))
    }

    pub fn validate_requirement(requirement: &mut i16) -> Result<()> {
        if *requirement < 0 || *requirement > 100 {
            return Err(PointercrateError::InvalidRequirement)
        }

        Ok(())
    }

    pub fn validate_name(name: &mut String, connection: &PgConnection) -> Result<()> {
        *name = name.trim().to_string();

        match Demon::get(name.as_ref(), connection) {
            Ok(demon) =>
                Err(PointercrateError::DemonExists {
                    position: demon.position,
                }),
            Err(PointercrateError::ModelNotFound { .. }) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub fn validate_position(position: &mut i16, connection: &PgConnection) -> Result<()> {
        let maximal = Demon::max_position(connection)? + 1;

        if *position < 1 || *position > maximal {
            return Err(PointercrateError::InvalidPosition { maximal })
        }

        Ok(())
    }

    pub fn validate_video(video: &mut String) -> Result<()> {
        *video = crate::video::validate(video)?;

        Ok(())
    }
}

impl Into<PartialDemon> for Demon {
    fn into(self) -> PartialDemon {
        PartialDemon {
            name: self.name,
            position: self.position,
            publisher: self.publisher.name,
        }
    }
}

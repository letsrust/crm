use std::{collections::HashSet, hash::Hash, time::Instant};

use anyhow::Result;
use chrono::{DateTime, Days, Utc};
use fake::{
    faker::chrono::en::DateTimeBetween, faker::internet::en::SafeEmail, faker::name::zh_cn::Name,
    Dummy, Fake, Faker,
};
use nanoid::nanoid;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

#[derive(Debug, Clone, Dummy, Serialize, Deserialize, PartialEq, Eq)]
struct UserStat {
    #[dummy(faker = "UniqueEmail")]
    email: String,
    #[dummy(faker = "Name()")]
    name: String,
    gender: Gender,
    #[dummy(faker = "DateTimeBetween(before(800), before(90))")]
    created_at: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(30), now())")]
    last_visited_at: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(90), now())")]
    last_watched_at: DateTime<Utc>,

    #[dummy(faker = "IntList(50, 100000, 100000)")]
    recent_watched: Vec<i32>,
    #[dummy(faker = "IntList(50, 200000, 100000)")]
    viewed_but_not_started: Vec<i32>,
    #[dummy(faker = "IntList(50, 300000, 100000)")]
    started_but_not_finished: Vec<i32>,
    #[dummy(faker = "IntList(50, 400000, 100000)")]
    finished: Vec<i32>,

    #[dummy(faker = "DateTimeBetween(before(45), now())")]
    last_email_notification: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(10), now())")]
    last_in_app_notification: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(90), now())")]
    last_sms_notification: DateTime<Utc>,
}

#[derive(Debug, Clone, Dummy, Serialize, Deserialize, PartialEq, Eq)]
enum Gender {
    F,
    M,
    U,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mypool = MySqlPool::connect("mysql://root:123456@127.0.0.1:3306/crm").await?;

    let f = Faker.fake::<UserStat>();
    println!("{f:?}");

    println!("---- {:?}", f.recent_watched);
    println!("***{}", list_to_string(f.recent_watched));

    for i in 1..=3 {
        let users: HashSet<_> = (0..1000).map(|_| Faker.fake::<UserStat>()).collect();

        let start = Instant::now();
        bulk_insert(users, &mypool).await?;
        println!("Batch {} cost {:?}", i, start.elapsed());
    }

    Ok(())
}

async fn bulk_insert(users: HashSet<UserStat>, pool: &MySqlPool) -> Result<()> {
    let mut sql = String::with_capacity(1024);
    sql.push_str("INSERT INTO user_stats(email, name, gender, created_at, last_visited_at, last_watched_at, recent_watched, viewed_but_not_started, started_but_not_finished, finished, last_email_notification, last_in_app_notification, last_sms_notification)
    VALUES");
    for user in users {
        sql.push_str(&format!(
            "('{}', '{}', '{:?}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}'),",
            user.email,
            user.name,
            user.gender,
            user.created_at.to_rfc3339(),
            user.last_visited_at.to_rfc3339(),
            user.last_watched_at.to_rfc3339(),
            list_to_string(user.recent_watched),
            list_to_string(user.viewed_but_not_started),
            list_to_string(user.started_but_not_finished),
            list_to_string(user.finished),
            user.last_email_notification.to_rfc3339(),
            user.last_in_app_notification.to_rfc3339(),
            user.last_sms_notification.to_rfc3339(),
        ));
    }

    let v = &sql[..sql.len() - 1];
    // println!("{v}");
    sqlx::query(v).execute(pool).await?;

    Ok(())
}

fn list_to_string(values: Vec<i32>) -> String {
    let r: String = values.iter().map(|&v| v.to_string() + ",").collect();
    let r_slice = match r.char_indices().next_back() {
        Some((i, _)) => &r[..i],
        None => r.as_str(),
    };

    String::from(r_slice)
}

// NaiveDateTime::parse_from_str(
//     user.created_at.to_string().as_str(),
//     "%Y-%m-%d %H:%M:%S"
// ),
fn before(days: u64) -> DateTime<Utc> {
    Utc::now().checked_sub_days(Days::new(days)).unwrap()
}

fn now() -> DateTime<Utc> {
    Utc::now()
}

impl Hash for UserStat {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.email.hash(state)
    }
}

struct IntList(pub i32, pub i32, pub i32);
impl Dummy<IntList> for Vec<i32> {
    fn dummy_with_rng<R: Rng + ?Sized>(v: &IntList, rng: &mut R) -> Vec<i32> {
        let (max, start, len) = (v.0, v.1, v.2);
        let size = rng.gen_range(0..max);
        (0..size)
            .map(|_| rng.gen_range(start..start + len))
            .collect()
    }
}

struct UniqueEmail;
const ALPHABET: [char; 36] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];
impl Dummy<UniqueEmail> for String {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &UniqueEmail, rng: &mut R) -> String {
        let email: String = SafeEmail().fake_with_rng(rng);
        let id = nanoid!(8, &ALPHABET);
        let at = email.find('@').unwrap();
        format!("{}.{}{}", &email[..at], id, &email[at..])
    }
}

use activitypub::{Actor, activity::{Accept, Follow as FollowAct}};
use diesel::{self, PgConnection, ExpressionMethods, QueryDsl, RunQueryDsl};

use activity_pub::{broadcast, Id, IntoId, inbox::{FromActivity, Notify, WithInbox}, sign::Signer};
use models::{
    blogs::Blog,
    notifications::*,
    users::User
};
use schema::follows;

#[derive(Queryable, Identifiable, Associations)]
#[belongs_to(User, foreign_key = "following_id")]
pub struct Follow {
    pub id: i32,
    pub follower_id: i32,
    pub following_id: i32
}

#[derive(Insertable)]
#[table_name = "follows"]
pub struct NewFollow {
    pub follower_id: i32,
    pub following_id: i32
}

impl Follow {
    insert!(follows, NewFollow);
    get!(follows);

    /// from -> The one sending the follow request
    /// target -> The target of the request, responding with Accept
    pub fn accept_follow<A: Signer + IntoId + Clone, B: Clone + WithInbox + Actor>(
        conn: &PgConnection,
        from: &B,
        target: &A,
        follow: FollowAct,
        from_id: i32,
        target_id: i32
    ) -> Follow {
        let res = Follow::insert(conn, NewFollow {
            follower_id: from_id,
            following_id: target_id
        });

        let mut accept = Accept::default();
        accept.accept_props.set_actor_link::<Id>(target.clone().into_id()).unwrap();
        accept.accept_props.set_object_object(follow).unwrap();
        broadcast(&*target, accept, vec![from.clone()]);
        res
    }
}

impl FromActivity<FollowAct> for Follow {
    fn from_activity(conn: &PgConnection, follow: FollowAct, _actor: Id) -> Follow {
        let from = User::from_url(conn, follow.follow_props.actor.as_str().unwrap().to_string()).unwrap();
        match User::from_url(conn, follow.follow_props.object.as_str().unwrap().to_string()) {
            Some(user) => Follow::accept_follow(conn, &from, &user, follow, from.id, user.id),
            None => {
                let blog = Blog::from_url(conn, follow.follow_props.object.as_str().unwrap().to_string()).unwrap();
                Follow::accept_follow(conn, &from, &blog, follow, from.id, blog.id)
            }
        }
    }
}

impl Notify for Follow {
    fn notify(&self, conn: &PgConnection) {
        let follower = User::get(conn, self.follower_id).unwrap();
        Notification::insert(conn, NewNotification {
            title: "{{ data }} started following you".to_string(),
            data: Some(follower.display_name.clone()),
            content: None,
            link: Some(follower.ap_url),
            user_id: self.following_id
        });
    }
}

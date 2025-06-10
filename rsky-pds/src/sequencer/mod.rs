use crate::account_manager::helpers::account::AccountStatus;
use crate::actor_store::repo::types::SyncEvtData;
use crate::crawlers::Crawlers;
use crate::db::establish_connection_for_sequencer;
use crate::models;
use crate::sequencer::events::{
    format_seq_account_evt, format_seq_commit, format_seq_handle_update, format_seq_identity_evt,
    SeqEvt, TypedAccountEvt, TypedCommitEvt, TypedIdentityEvt, TypedSyncEvt,
};
use crate::EVENT_EMITTER;
use anyhow::Result;
use diesel::*;
use events::format_seq_sync_evt;
use futures::{Stream, StreamExt};
use rsky_common::time::SECOND;
use rsky_common::{cbor_to_struct, wait};
use rsky_repo::types::CommitDataWithOps;
use std::cmp;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

pub struct RequestSeqRangeOpts {
    pub earliest_seq: Option<i64>,
    pub latest_seq: Option<i64>,
    pub earliest_time: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct Sequencer {
    pub destroyed: bool,
    pub tries_with_no_results: u32,
    pub waker: Option<Waker>,
    pub crawlers: Crawlers,
    pub last_seen: Option<i64>,
}

impl Sequencer {
    pub fn new(crawlers: Crawlers, last_seen: Option<i64>) -> Self {
        Sequencer {
            destroyed: false,
            tries_with_no_results: 0,
            last_seen: Some(last_seen.unwrap_or(0)),
            waker: None,
            crawlers,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let curr = self.curr().await?;
        self.last_seen = Some(curr.unwrap_or(0));
        if self.waker.is_none() {
            loop {
                while let Some(_) = self.next().await {}
            }
        }
        Ok(())
    }

    pub async fn destroy(&mut self) {
        self.destroyed = true;
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
        EVENT_EMITTER.write().await.emit("close", ());
    }

    pub async fn curr(&self) -> Result<Option<i64>> {
        use crate::schema::pds::repo_seq::dsl as RepoSeqSchema;
        let pool = establish_connection_for_sequencer()?;

        let got = pool.get().await?.interact(move |conn| {
            RepoSeqSchema::repo_seq
                .select(models::RepoSeq::as_select())
                .order_by(RepoSeqSchema::seq.desc())
                .first(conn)
                .optional()
        }).await.expect("Failed to get current sequence")?;
        match got {
            None => Ok(None),
            Some(got) => Ok(got.seq),
        }
    }

    pub async fn next_seq(&self, cursor: i64) -> Result<Option<models::RepoSeq>> {
        use crate::schema::pds::repo_seq::dsl as RepoSeqSchema;
        let pool = establish_connection_for_sequencer()?;

        let got = pool.get().await?.interact(move |conn| {
            RepoSeqSchema::repo_seq
                .filter(RepoSeqSchema::seq.gt(cursor))
                .select(models::RepoSeq::as_select())
                .order_by(RepoSeqSchema::seq.asc())
                .first(conn)
                .optional()
        }).await.expect("Failed to get next sequence")?;
        Ok(got)
    }

    pub async fn earliest_after_time(&self, time: String) -> Result<Option<models::RepoSeq>> {
        use crate::schema::pds::repo_seq::dsl as RepoSeqSchema;
        let pool = establish_connection_for_sequencer()?;

        let time_clone = time.clone();
        let got = pool.get().await?.interact(move |conn| {
            RepoSeqSchema::repo_seq
                .filter(RepoSeqSchema::sequencedAt.ge(time_clone))
                .select(models::RepoSeq::as_select())
                .order_by(RepoSeqSchema::sequencedAt.asc())
                .first(conn)
                .optional()
        }).await.expect("Failed to get earliest after time")?;
        Ok(got)
    }

    pub async fn request_seq_range(&self, opts: RequestSeqRangeOpts) -> Result<Vec<SeqEvt>> {
        use crate::schema::pds::repo_seq::dsl as RepoSeqSchema;
        let pool = establish_connection_for_sequencer()?;

        let RequestSeqRangeOpts {
            earliest_seq,
            latest_seq,
            earliest_time,
            limit,
        } = opts;

        let rows = pool.get().await?.interact(move |conn| {
            let mut seq_qb = RepoSeqSchema::repo_seq
                .select(models::RepoSeq::as_select())
                .order_by(RepoSeqSchema::seq.asc())
                .filter(RepoSeqSchema::invalidated.eq(0))
                .into_boxed();
            if let Some(earliest_seq) = earliest_seq {
                seq_qb = seq_qb.filter(RepoSeqSchema::seq.gt(earliest_seq));
            }
            if let Some(latest_seq) = latest_seq {
                seq_qb = seq_qb.filter(RepoSeqSchema::seq.le(latest_seq));
            }
            if let Some(earliest_time) = earliest_time {
                seq_qb = seq_qb.filter(RepoSeqSchema::sequencedAt.ge(earliest_time));
            }
            if let Some(limit) = limit {
                seq_qb = seq_qb.limit(limit);
            }

            seq_qb.get_results(conn)
        }).await.expect("Failed to request sequence range")?;

        if rows.is_empty() {
            return Ok(vec![]);
        }

        let mut seq_evts: Vec<SeqEvt> = Vec::new();
        for row in rows {
            let time = row.sequenced_at;
            match row.seq {
                None => continue, // should never hit this because of WHERE clause
                Some(seq) => match row.event_type.as_str() {
                    "append" | "rebase" => {
                        seq_evts.push(SeqEvt::TypedCommitEvt(TypedCommitEvt {
                            r#type: "commit".to_string(),
                            seq,
                            time,
                            evt: cbor_to_struct(row.event)?,
                        }));
                    }
                    "sync" => {
                        seq_evts.push(SeqEvt::TypedSyncEvt(TypedSyncEvt {
                            r#type: "sync".to_string(),
                            seq,
                            time,
                            evt: cbor_to_struct(row.event)?,
                        }));
                    }
                    "identity" => {
                        seq_evts.push(SeqEvt::TypedIdentityEvt(TypedIdentityEvt {
                            r#type: "identity".to_string(),
                            seq,
                            time,
                            evt: cbor_to_struct(row.event)?,
                        }));
                    }
                    "account" => {
                        seq_evts.push(SeqEvt::TypedAccountEvt(TypedAccountEvt {
                            r#type: "account".to_string(),
                            seq,
                            time,
                            evt: cbor_to_struct(row.event)?,
                        }));
                    }
                    _ => {
                        eprintln!("ERROR: request_seq_range invalid event type");
                    }
                },
            }
        }

        Ok(seq_evts)
    }

    async fn exponential_backoff(&mut self) {
        self.tries_with_no_results += 1;
        let wait_time = cmp::min(
            2u64.checked_pow(self.tries_with_no_results).unwrap_or(2),
            SECOND as u64,
        );
        wait(wait_time);
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }

    pub async fn sequence_evt(&mut self, evt: models::RepoSeq) -> Result<i64> {
        use crate::schema::pds::repo_seq::dsl as RepoSeqSchema;
        let pool = establish_connection_for_sequencer()?;

        let did = evt.did;
        let event = evt.event;
        let event_type = evt.event_type;
        let sequenced_at = evt.sequenced_at;

        let res = pool.get().await?.interact(move |conn| {
            insert_into(RepoSeqSchema::repo_seq)
                .values((
                    RepoSeqSchema::did.eq(did),
                    RepoSeqSchema::event.eq(event),
                    RepoSeqSchema::eventType.eq(event_type),
                    RepoSeqSchema::sequencedAt.eq(sequenced_at),
                ))
                .get_result::<models::RepoSeq>(conn)
        }).await.expect("Failed to sequence event")?;
        
        self.crawlers.notify_of_update().await?;
        Ok(res.seq.expect("Sequence number wasn't updated on insert."))
    }

    pub async fn sequence_commit(
        &mut self,
        did: String,
        commit_data: CommitDataWithOps,
    ) -> Result<i64> {
        let evt = format_seq_commit(did, commit_data).await?;
        self.sequence_evt(evt).await
    }

    pub async fn sequence_handle_update(&mut self, did: String, handle: String) -> Result<i64> {
        let evt = format_seq_handle_update(did, handle).await?;
        self.sequence_evt(evt).await
    }

    pub async fn sequence_identity_evt(
        &mut self,
        did: String,
        handle: Option<String>,
    ) -> Result<i64> {
        let evt = format_seq_identity_evt(did, handle).await?;
        self.sequence_evt(evt).await
    }

    pub async fn sequence_account_evt(
        &mut self,
        did: String,
        status: AccountStatus,
    ) -> Result<i64> {
        let evt = format_seq_account_evt(did, status).await?;
        self.sequence_evt(evt).await
    }

    pub async fn sequence_sync_evt(&mut self, did: String, data: SyncEvtData) -> Result<i64> {
        let evt = format_seq_sync_evt(did, data).await?;
        self.sequence_evt(evt).await
    }
}

impl Stream for Sequencer {
    type Item = Result<(), anyhow::Error>;

    #[tracing::instrument(skip_all)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.destroyed {
            return Poll::Ready(None);
        }
        
        // Store the waker for future notifications
        self.waker = Some(cx.waker().clone());
        
        // If already polling, do not start another poll
        let opts = RequestSeqRangeOpts {
            earliest_seq: self.last_seen,
            latest_seq: None,
            earliest_time: None,
            limit: Some(1000),
        };
        
        // Use a ready future to drive our async function - this avoids nesting block_on calls
        match futures::executor::block_on(self.request_seq_range(opts)) {
            Err(err) => {
                tracing::error!(
                    "@LOG: sequencer failed to poll db, err: {}, last_seen: {:?}",
                    err.to_string(),
                    self.last_seen
                );
                futures::executor::block_on(self.exponential_backoff());
                Poll::Ready(Some(Err(err)))
            }
            Ok(evts) => {
                if !evts.is_empty() {
                    self.tries_with_no_results = 0;
                    futures::executor::block_on(EVENT_EMITTER.write()).emit(
                        "events",
                        evts.iter()
                            .map(|evt| serde_json::to_string(evt).unwrap())
                            .collect::<Vec<String>>(),
                    );
                    self.last_seen = match evts.last() {
                        None => self.last_seen,
                        Some(last_evt) => Some(last_evt.seq()),
                    };
                    Poll::Ready(Some(Ok(())))
                } else {
                    futures::executor::block_on(self.exponential_backoff());
                    Poll::Pending
                }
            }
        }
    }
}

pub async fn delete_all_for_user(did: &String, excluding_seqs: Option<Vec<i64>>) -> Result<()> {
    use crate::schema::pds::repo_seq::dsl as RepoSeqSchema;
    let pool = establish_connection_for_sequencer()?;
    let excluding_seqs = excluding_seqs.unwrap_or_else(|| vec![]);
    let did_clone = did.clone();

    pool.get().await?.interact(move |conn| {
        let mut builder = delete(RepoSeqSchema::repo_seq)
            .filter(RepoSeqSchema::did.eq(did_clone))
            .into_boxed();
        if !excluding_seqs.is_empty() {
            builder = builder.filter(RepoSeqSchema::seq.ne_all(excluding_seqs));
        }
        builder.execute(conn)
    }).await.expect("Failed to delete sequences for user")?;
    
    Ok(())
}

pub mod events;
pub mod outbox;

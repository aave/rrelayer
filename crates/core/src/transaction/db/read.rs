use super::builders::build_transaction_from_transaction_view;
use crate::{
    postgres::{PostgresClient, PostgresError},
    relayer::RelayerId,
    shared::common_types::{PagingContext, PagingResult},
    transaction::types::{Transaction, TransactionHash, TransactionId, TransactionStatus},
};

impl PostgresClient {
    pub async fn get_transaction(
        &self,
        id: &TransactionId,
    ) -> Result<Option<Transaction>, PostgresError> {
        let row = self
            .query_one_or_none(
                "
                    SELECT *
                    FROM relayer.transaction
                    WHERE id = $1;
                ",
                &[id],
            )
            .await?;

        match row {
            None => Ok(None),
            Some(row) => Ok(Some(build_transaction_from_transaction_view(&row))),
        }
    }

    pub async fn get_transactions_for_relayer(
        &self,
        id: &RelayerId,
        paging_context: &PagingContext,
    ) -> Result<PagingResult<Transaction>, PostgresError> {
        let rows = self
            .query(
                "
                    SELECT *
                    FROM relayer.transaction
                    WHERE relayer_id = $1
                    LIMIT $2
                    OFFSET $3;
                ",
                &[&id, &(paging_context.limit as i64), &(paging_context.offset as i64)],
            )
            .await?;

        let results: Vec<Transaction> =
            rows.iter().map(build_transaction_from_transaction_view).collect();

        let result_count = results.len();

        Ok(PagingResult::new(results, paging_context.next(result_count), paging_context.previous()))
    }

    pub async fn get_transactions_by_status_for_relayer(
        &self,
        id: &RelayerId,
        status: &TransactionStatus,
        paging_context: &PagingContext,
    ) -> Result<PagingResult<Transaction>, PostgresError> {
        let rows = self
            .query(
                "
                    SELECT *
                    FROM relayer.transaction
                    WHERE relayer_id = $1
                    AND status = $2
                    ORDER BY nonce ASC
                    LIMIT $3
                    OFFSET $4;
                ",
                &[id, status, &(paging_context.limit as i64), &(paging_context.offset as i64)],
            )
            .await?;

        let results: Vec<Transaction> =
            rows.iter().map(build_transaction_from_transaction_view).collect();

        let result_count = results.len();

        Ok(PagingResult::new(results, paging_context.next(result_count), paging_context.previous()))
    }

    /// Returns every distinct hash this transaction has ever been broadcast (or
    /// prepared for broadcast) with, from the audit log.
    ///
    /// Gas bumps and send retries re-sign the same payload, so a transaction can
    /// have one hash per attempt and any of them may be the one that mined.
    ///
    /// # Examples
    ///
    /// Find which attempt of a transaction actually landed on-chain:
    ///
    /// ```no_run
    /// use rrelayer_core::PostgresClient;
    /// use rrelayer_core::transaction::types::{TransactionHash, TransactionId};
    ///
    /// async fn candidate_hashes(
    ///     db: &PostgresClient,
    ///     transaction_id: &TransactionId,
    /// ) -> Result<Vec<TransactionHash>, Box<dyn std::error::Error>> {
    ///     let attempts = db.transaction_attempt_hashes(transaction_id).await?;
    ///     Ok(attempts)
    /// }
    /// ```
    pub async fn transaction_attempt_hashes(
        &self,
        transaction_id: &TransactionId,
    ) -> Result<Vec<TransactionHash>, PostgresError> {
        let rows = self
            .query(
                "
                    SELECT DISTINCT hash
                    FROM relayer.transaction_audit_log
                    WHERE id = $1 AND hash IS NOT NULL;
                ",
                &[transaction_id],
            )
            .await?;

        Ok(rows.iter().map(|row| row.get("hash")).collect())
    }

    pub async fn get_transaction_by_hash(
        &self,
        hash: &TransactionHash,
    ) -> Result<Option<Transaction>, PostgresError> {
        let row = self
            .query_one_or_none(
                "
                    SELECT *
                    FROM relayer.transaction
                    WHERE hash = $1;
                ",
                &[hash],
            )
            .await?;

        match row {
            None => Ok(None),
            Some(row) => Ok(Some(build_transaction_from_transaction_view(&row))),
        }
    }

    pub async fn get_transaction_by_external_id(
        &self,
        external_id: &str,
    ) -> Result<Option<Transaction>, PostgresError> {
        let row = self
            .query_one_or_none(
                "
                    SELECT *
                    FROM relayer.transaction
                    WHERE external_id = $1;
                ",
                &[&external_id],
            )
            .await?;

        match row {
            None => Ok(None),
            Some(row) => Ok(Some(build_transaction_from_transaction_view(&row))),
        }
    }

    pub async fn get_transaction_by_relayer_and_external_id(
        &self,
        relayer_id: &RelayerId,
        external_id: &str,
    ) -> Result<Option<Transaction>, PostgresError> {
        let row = self
            .query_one_or_none(
                "
                    SELECT *
                    FROM relayer.transaction
                    WHERE relayer_id = $1
                    AND external_id = $2;
                ",
                &[relayer_id, &external_id],
            )
            .await?;

        match row {
            None => Ok(None),
            Some(row) => Ok(Some(build_transaction_from_transaction_view(&row))),
        }
    }
}

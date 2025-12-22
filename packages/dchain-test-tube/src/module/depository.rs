use dchain_sdk_proto::{
    dchain::depository::v1::{
        MsgIssuePtWithGlobalNote, MsgIssuePtWithGlobalNoteResponse, MsgSurrenderGlobalNote,
        MsgSurrenderGlobalNoteResponse, QueryGetGlobalNoteByIsinRequest,
        QueryGetGlobalNoteByIsinResponse,
    },
    traits::TypeUrl,
};

use test_tube::runner::Runner;
use test_tube::{fn_execute, fn_query};
pub struct Depository<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R> Depository<'a, R>
where
    R: Runner<'a>,
{
    fn_execute! {
        pub issue_pt_with_global_note: MsgIssuePtWithGlobalNote => MsgIssuePtWithGlobalNoteResponse
    }

    fn_execute! {
        pub surrender_global_note: MsgSurrenderGlobalNote => MsgSurrenderGlobalNoteResponse
    }

    fn_query! {
        pub get_global_note_by_isin ["/d.depository.v1.Query/GetGlobalNoteByIsin"]: QueryGetGlobalNoteByIsinRequest => QueryGetGlobalNoteByIsinResponse
    }
}

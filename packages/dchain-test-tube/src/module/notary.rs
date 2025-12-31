use dchain_sdk_proto::dchain::notary::v1::{
    GetCurrencyConversionRateRequest, GetCurrencyConversionRateResponse, GetNotarisedAssetRequest,
    GetNotarisedAssetResponse, GetNotaryInfoByIdRequest, GetNotaryInfoByIdResponse, MsgNotarise,
    MsgNotariseResponse, MsgRegisterNotaryInfo, MsgRegisterNotaryInfoResponse, MsgRemoveNotaryInfo,
    MsgRemoveNotaryInfoResponse, MsgUpdateAdmin, MsgUpdateAdminResponse, MsgUpdateNotarisedAsset,
    MsgUpdateNotarisedAssetResponse, MsgUpdateVerifierRoutes, MsgUpdateVerifierRoutesResponse,
};
use dchain_sdk_proto::traits::TypeUrl;

use test_tube::{fn_execute, fn_query};

use test_tube::runner::Runner;

pub struct Notary<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> super::Module<'a, R> for Notary<'a, R> {
    fn new(runner: &'a R) -> Self {
        Notary { runner }
    }
}

impl<'a, R> Notary<'a, R>
where
    R: Runner<'a>,
{
    fn_execute! {
        pub notarise: MsgNotarise => MsgNotariseResponse
    }

    fn_execute! {
        pub register_notary_info: MsgRegisterNotaryInfo => MsgRegisterNotaryInfoResponse
    }

    fn_execute! {
        pub remove_notary_info: MsgRemoveNotaryInfo => MsgRemoveNotaryInfoResponse
    }

    fn_execute! {
        pub update_admin: MsgUpdateAdmin => MsgUpdateAdminResponse
    }

    fn_execute! {
        pub update_notarised_asset: MsgUpdateNotarisedAsset => MsgUpdateNotarisedAssetResponse
    }

    fn_execute! {
        pub update_verifier_routes: MsgUpdateVerifierRoutes => MsgUpdateVerifierRoutesResponse
    }

    fn_query! {
        pub get_notarised_asset ["/d.notary.v1.Query/GetNotarisedAsset"]: GetNotarisedAssetRequest => GetNotarisedAssetResponse
    }

    fn_query! {
        pub get_notary_info_by_id ["/d.notary.v1.Query/GetNotaryInfoById"]: GetNotaryInfoByIdRequest => GetNotaryInfoByIdResponse
    }

    fn_query! {
        pub get_conversion_rate ["/d.notary.v1.Query/GetCurrencyConversionRate"]: GetCurrencyConversionRateRequest => GetCurrencyConversionRateResponse
    }
}

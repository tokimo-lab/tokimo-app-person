//! ts-rs type export — run with `cargo test -p tokimo-app-person -- export_bindings`.
//! Generates TypeScript types to `ui/src/generated/rust-types/`.

use tokimo_app_person::handlers::{
    DeleteSourceResponse, FaceDetailDto, MatchFaceResponse, PersonDetailDto, PersonDto, PersonListResponse,
    RegisterFacesResponse, SourceMediaDto, UpdatePersonReq,
};
use ts_rs::{Config, TS};

#[test]
fn export_bindings() {
    let cfg = Config::from_env();

    PersonDto::export_all(&cfg).unwrap();
    PersonListResponse::export_all(&cfg).unwrap();
    FaceDetailDto::export_all(&cfg).unwrap();
    SourceMediaDto::export_all(&cfg).unwrap();
    PersonDetailDto::export_all(&cfg).unwrap();
    UpdatePersonReq::export_all(&cfg).unwrap();
    RegisterFacesResponse::export_all(&cfg).unwrap();
    MatchFaceResponse::export_all(&cfg).unwrap();
    DeleteSourceResponse::export_all(&cfg).unwrap();
}

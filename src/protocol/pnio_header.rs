use super::{ArBlockReq, ArBlockRes, IodReq, IodRes};

pub trait PnioHeader {
    fn concat(&self) -> anyhow::Result<Vec<u8>>;
    fn size(&self) -> usize;
    fn get_max_count(&self) -> u32;
    fn get_actual_count(&self) -> u32;
    fn get_args_length(&self) -> u32;
    fn get_args_max(&self) -> Option<u32>;
}

#[derive(Debug)]
pub enum PnioHeaderEnum {
    ArBlockReq(ArBlockReq),
    ArBlockRes(ArBlockRes),
    IodReq(IodReq),
    IodRes(IodRes),
}

impl PnioHeader for PnioHeaderEnum {
    fn concat(&self) -> anyhow::Result<Vec<u8>> {
        match self {
            PnioHeaderEnum::ArBlockReq(p) => p.concat(),
            PnioHeaderEnum::ArBlockRes(p) => p.concat(),
            PnioHeaderEnum::IodReq(p) => p.concat(),
            PnioHeaderEnum::IodRes(p) => p.concat(),
        }
    }

    fn size(&self) -> usize {
        match self {
            PnioHeaderEnum::ArBlockReq(p) => p.size(),
            PnioHeaderEnum::ArBlockRes(p) => p.size(),
            PnioHeaderEnum::IodReq(p) => p.size(),
            PnioHeaderEnum::IodRes(p) => p.size(),
        }
    }

    fn get_max_count(&self) -> u32 {
        match self {
            PnioHeaderEnum::ArBlockReq(p) => p.get_max_count(),
            PnioHeaderEnum::ArBlockRes(p) => p.get_max_count(),
            PnioHeaderEnum::IodReq(p) => p.get_max_count(),
            PnioHeaderEnum::IodRes(p) => p.get_max_count(),
        }
    }

    fn get_actual_count(&self) -> u32 {
        match self {
            PnioHeaderEnum::ArBlockReq(p) => p.get_actual_count(),
            PnioHeaderEnum::ArBlockRes(p) => p.get_actual_count(),
            PnioHeaderEnum::IodReq(p) => p.get_actual_count(),
            PnioHeaderEnum::IodRes(p) => p.get_actual_count(),
        }
    }

    fn get_args_length(&self) -> u32 {
        match self {
            PnioHeaderEnum::ArBlockReq(p) => p.get_args_length(),
            PnioHeaderEnum::ArBlockRes(p) => p.get_args_length(),
            PnioHeaderEnum::IodReq(p) => p.get_args_length(),
            PnioHeaderEnum::IodRes(p) => p.get_args_length(),
        }
    }

    fn get_args_max(&self) -> Option<u32> {
        match self {
            PnioHeaderEnum::ArBlockReq(p) => p.get_args_max(),
            PnioHeaderEnum::ArBlockRes(p) => p.get_args_max(),
            PnioHeaderEnum::IodReq(p) => p.get_args_max(),
            PnioHeaderEnum::IodRes(p) => p.get_args_max(),
        }
    }
}

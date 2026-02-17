// 组件模块 - 用于放置可复用的UI组件

pub mod dialog;

pub use dialog::{
    ConfirmDialog, ErrorDialog, LoadingDialog, QuitConfirmDialog, SuccessDialog, WelcomeDialog,
};

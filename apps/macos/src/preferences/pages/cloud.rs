//! 「云服务」页：本地整句模型开关，云联想开关、云端词格数、接口地址 / 模型 / 密钥、测试连接。

use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{NSButton, NSPopUpButton, NSSecureTextField, NSTextField};
use objc2_foundation::NSString;
use qingjian_platform::Config;

use crate::preferences::controls::{
    button, checkbox, note, plain_secure_field, plain_text_field, row_checkbox, row_control,
    row_popup, secure_field, select, set_checked, text_field,
};
use crate::preferences::layout::{Layout, PAGE_PADDING, ROW_HEIGHT};
use crate::preferences::setting::Setting;
use crate::preferences::target::PreferencesTarget;

/// 云端词槽位弹出菜单的上限（配置文件里可以填更大，菜单只列到这）。
const MAX_CLOUD_SLOTS: usize = 4;

/// Cloudflare 控制台「Workers AI」页深链：登录后直接到当前账号的 Workers AI 页，
/// 「Use REST API → Create a Workers AI API Token」一步能同时拿到账号 ID 与预置读写权限的 API 令牌。
pub const CLOUDFLARE_CONSOLE_URL: &str = "https://dash.cloudflare.com/?to=/:account/ai/workers-ai";

/// 免费额度里延迟较低的小模型，「填入以上三项」用它起步，用户仍可在「模型」框里改。
pub const CLOUDFLARE_DEFAULT_MODEL: &str = "@cf/meta/llama-3.2-3b-instruct";

pub struct CloudPage {
    /// 本地整句模型开关。
    local_model: Retained<NSButton>,

    /// 云联想开关。
    enabled: Retained<NSButton>,

    /// 云端词槽位数（0–4）。
    slots: Retained<NSPopUpButton>,

    /// 接口地址。
    base_url: Retained<NSTextField>,

    /// 模型名。
    model: Retained<NSTextField>,

    /// 密钥输入框，永远不回显已有值。
    api_key: Retained<NSSecureTextField>,

    /// 「测试连接」按钮。
    test: Retained<NSButton>,

    /// 「打开 Cloudflare 获取密钥」按钮。
    cf_open: Retained<NSButton>,

    /// Cloudflare 账号 ID 临时输入框，只读一次拼 `base_url`，不落盘、不接 `Setting`。
    cf_account_id: Retained<NSTextField>,

    /// Cloudflare API 令牌临时输入框，同上。
    cf_token: Retained<NSSecureTextField>,

    /// 「填入以上三项」按钮。
    cf_fill: Retained<NSButton>,
}

impl CloudPage {
    pub fn build(layout: &mut Layout, mtm: MainThreadMarker, target: &PreferencesTarget) -> Self {
        let local_model = checkbox(mtm, "本地整句模型", Setting::LocalModelEnabled, target);
        row_checkbox(layout, &local_model);
        note(
            layout,
            mtm,
            "随包的小模型在本机给整句候选重新排序，全程离线；停键后几十毫秒生效。关掉只用词库统计。",
        );
        let enabled = checkbox(mtm, "启用云联想", Setting::CloudEnabled, target);
        row_checkbox(layout, &enabled);
        note(
            layout,
            mtm,
            "开启后组句时会把光标附近的几十个字发给下面的服务，让模型补全整句、联想下文；密码框里绝不发送。菜单栏图标旁会带一个云朵。",
        );
        let slot_titles: Vec<String> = (0..=MAX_CLOUD_SLOTS)
            .map(|n| match n {
                0 => "不要（只要整句补全）".to_owned(),
                n => format!("{n} 格"),
            })
            .collect();
        let slots = row_popup(
            layout,
            mtm,
            "云端词位置",
            &slot_titles,
            Setting::CloudSlots,
            target,
        );
        note(
            layout,
            mtm,
            "云端词到了只补进第一页末尾这几格（比如 2 就是 8、9），前面的本地候选不动；没到就什么都不变，翻页后全是本地候选。",
        );
        let base_url = text_field(mtm, Setting::BaseUrl, target);
        row_control(layout, mtm, "接口地址", &base_url);
        let model = text_field(mtm, Setting::Model, target);
        row_control(layout, mtm, "模型", &model);
        let api_key = secure_field(mtm, Setting::ApiKey, target);
        row_control(layout, mtm, "API 密钥", &api_key);
        note(
            layout,
            mtm,
            "文本框按回车保存。密钥只保存在这台电脑上，不会随配置文件导出，也不显示已填的值。",
        );
        let test = button(mtm, "测试连接", Setting::TestCloud, target);
        layout.place(&test, PAGE_PADDING, 120.0, ROW_HEIGHT + 4.0);
        layout.next_row(ROW_HEIGHT + 4.0);
        note(
            layout,
            mtm,
            "用上面填的地址、模型、密钥发一条最小请求，结果显示在窗口底部。输入法进程看不到终端里的代理变量，走不通时先查这个。",
        );

        let cf_open = button(
            mtm,
            "打开 Cloudflare 获取密钥",
            Setting::OpenCloudflareConsole,
            target,
        );
        layout.place(&cf_open, PAGE_PADDING, 220.0, ROW_HEIGHT + 4.0);
        layout.next_row(ROW_HEIGHT + 4.0);
        note(
            layout,
            mtm,
            "没有服务商时可用 Cloudflare Workers AI 的免费额度试用：打开后在「Workers AI」页点「Use REST API」，",
        );
        note(
            layout,
            mtm,
            "账号 ID 与预置好权限的 API 令牌会一起显示，复制粘贴到下面两项，再点「填入以上三项」。",
        );
        let cf_account_id = plain_text_field(mtm);
        row_control(layout, mtm, "Cloudflare 账号 ID", &cf_account_id);
        let cf_token = plain_secure_field(mtm);
        row_control(layout, mtm, "Cloudflare API 令牌", &cf_token);
        let cf_fill = button(mtm, "填入以上三项", Setting::CloudflareFill, target);
        layout.place(&cf_fill, PAGE_PADDING, 160.0, ROW_HEIGHT + 4.0);
        layout.next_row(ROW_HEIGHT + 4.0);

        Self {
            local_model,
            enabled,
            slots,
            base_url,
            model,
            api_key,
            test,
            cf_open,
            cf_account_id,
            cf_token,
            cf_fill,
        }
    }

    /// `key_present` 是密钥已经有了（环境或配置里）；密钥框永远不回显值，只换占位文字。
    /// `model_present` 是包里或用户目录里有模型文件，没有就把本地模型的勾选灰掉；云联想关着时它下面的项全灰。
    pub fn sync(&self, config: &Config, key_present: bool, model_present: bool) {
        set_checked(&self.local_model, config.model.enabled && model_present);
        self.local_model.setEnabled(model_present);
        set_checked(&self.enabled, config.predict.enabled);
        let cloud = config.predict.enabled;
        self.slots.setEnabled(cloud);
        self.base_url.setEnabled(cloud);
        self.model.setEnabled(cloud);
        self.api_key.setEnabled(cloud);
        self.test.setEnabled(cloud);
        self.cf_open.setEnabled(cloud);
        self.cf_account_id.setEnabled(cloud);
        self.cf_token.setEnabled(cloud);
        self.cf_fill.setEnabled(cloud);
        select(&self.slots, Some(config.predict.slots.min(MAX_CLOUD_SLOTS)));
        self.base_url
            .setStringValue(&NSString::from_str(&config.predict.base_url));
        self.model
            .setStringValue(&NSString::from_str(&config.predict.model));
        self.api_key.setStringValue(&NSString::from_str(""));
        let hint = if key_present {
            "已设置，输入新值可替换"
        } else {
            "未设置"
        };
        self.api_key
            .setPlaceholderString(Some(&NSString::from_str(hint)));
    }

    /// 「账号 ID」「API 令牌」两个临时框现在的值，`change_setting` 处理 `CloudflareFill` 时读一次。
    pub fn cloudflare_credentials(&self) -> (String, String) {
        (
            self.cf_account_id.stringValue().to_string(),
            self.cf_token.stringValue().to_string(),
        )
    }

    /// 填完就清空，令牌不多停留在这个次要输入框里。
    pub fn clear_cloudflare_credentials(&self) {
        self.cf_account_id.setStringValue(&NSString::from_str(""));
        self.cf_token.setStringValue(&NSString::from_str(""));
    }
}

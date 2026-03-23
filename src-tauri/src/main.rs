// 防止 Windows 系统显示控制台窗口
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod models;
mod utils;

use commands::{agents, chat, config, diagnostics, installer, process, prompt, sandbox, service, skills, updater};

fn main() {
    // 初始化日志 - 默认显示 info 级别日志
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();
    
    log::info!("🦞 OTOClaw 启动");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // 服务管理
            service::get_service_status,
            service::start_service,
            service::stop_service,
            service::restart_service,
            service::get_logs,
            // 进程管理
            process::check_openclaw_installed,
            process::get_openclaw_version,
            process::check_port_in_use,
            // 配置管理
            config::get_config,
            config::save_config,
            config::get_env_value,
            config::save_env_value,
            config::get_ai_providers,
            config::get_channels_config,
            config::save_channel_config,
            config::clear_channel_config,
            // Gateway Token
            config::get_or_create_gateway_token,
            config::get_dashboard_url,
            // AI 配置管理
            config::get_official_providers,
            config::get_ai_config,
            config::save_provider,
            config::delete_provider,
            config::set_primary_model,
            config::add_available_model,
            config::remove_available_model,
            // 飞书插件管理
            config::check_feishu_plugin,
            config::install_feishu_plugin,
            // 聊天模块
            config::get_agents,
            config::get_models,
            config::get_gateway_config,
            config::save_gateway_config,
            config::init_gateway_token,
            // 聊天会话管理
            chat::get_sessions,
            chat::get_session_messages,
            chat::connect_gateway,
            chat::disconnect_gateway,
            chat::create_session,
            chat::delete_session,
            chat::send_chat_message,
            chat::check_gateway_status,
            // 智能体管理
            agents::get_agents_list,
            agents::create_agent,
            agents::update_agent,
            agents::delete_agent,
            agents::set_default_agent,
            agents::get_agent_by_id,
            agents::get_agent_bindings,
            agents::set_agent_bindings,
            agents::get_available_channels,
            agents::get_agent_workspace_files,
            agents::save_agent_workspace_file,
            // 技能管理
            skills::get_skills_list,
            skills::get_builtin_skills,
            skills::check_skill_requirements,
            skills::get_skill_detail,
            skills::create_skill,
            skills::update_skill_config,
            skills::delete_skill,
            skills::install_skill_from_zip,
            skills::export_skill,
            skills::open_skill_directory,
            skills::install_skill_dependency,
            skills::get_agent_skills,
            skills::assign_skill_to_agent,
            skills::remove_skill_from_agent,
            // 诊断测试
            diagnostics::run_doctor,
            diagnostics::test_ai_connection,
            diagnostics::test_channel,
            diagnostics::get_system_info,
            diagnostics::start_channel_login,
            diagnostics::run_openclaw_fix,
            // 安装器
            installer::check_environment,
            installer::install_nodejs,
            installer::install_openclaw,
            installer::init_openclaw_config,
            installer::open_install_terminal,
            installer::uninstall_openclaw,
            // 版本更新
            installer::check_openclaw_update,
            installer::update_openclaw,
            // OTOClaw 更新
            updater::get_update_config,
            updater::save_update_config_cmd,
            updater::check_otoclaw_update,
            updater::download_update,
            updater::install_update,
            updater::get_app_version,
            updater::skip_version,
            updater::cancel_update,
            // 沙箱管理
            sandbox::get_sandbox_status,
            sandbox::get_sandbox_config,
            sandbox::save_sandbox_config,
            sandbox::validate_sandbox_config_cmd,
            sandbox::list_sandbox_containers,
            sandbox::stop_sandbox_container,
            sandbox::remove_sandbox_container,
            sandbox::prune_sandbox_containers,
            sandbox::recreate_sandbox_container,
            sandbox::check_docker_available_cmd,
            sandbox::get_docker_version_cmd,
            sandbox::pull_sandbox_image,
            sandbox::build_sandbox_image,
            // 提示词优化
            prompt::optimize_prompt,
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时发生错误");
}

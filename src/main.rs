//! inR Remapper - Windows 键盘重映射工具
//! 
//! 使用 Windows Low-Level Keyboard Hook 实现按键拦截与重映射

mod config;
mod hook;
mod mapping;

fn main() {
    println!("==============================");
    println!("  inR Remapper");
    println!("==============================");
    println!();

    // 读取配置
    let cfg = match config::load("config.json") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[错误] 无法读取配置: {}", e);
            std::process::exit(1);
        }
    };

    if cfg.mappings.is_empty() {
        println!("[提示] 没有配置任何映射");
    } else {
        println!("[配置] 加载按键映射:");
        for (from, to) in &cfg.mappings {
            println!("  {} → {}", from, to);
        }
        println!();
    }

    // 创建映射表
    let map = mapping::create_mapping(&cfg);
    
    // 设置映射表到钩子模块
    hook::set_mapping(map);

    // 初始化键盘钩子
    if let Err(e) = hook::init() {
        eprintln!("[错误] {}", e);
        eprintln!("提示: 可能需要管理员权限");
        std::process::exit(1);
    }

    println!("[运行] 按 Ctrl+C 退出\n");

    // 设置 Ctrl+C 处理器
    ctrlc::set_handler(|| {
        println!("\n[退出] 正在停止...");
        hook::stop();
        hook::cleanup();
        std::process::exit(0);
    }).expect("ctrlc handler failed");

    // 运行消息循环
    hook::run_message_loop();
}

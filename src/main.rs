use std::{env, fs};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use clap::{Parser};
use winit::{
    dpi::LogicalSize,
    event::{Event},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};
use pixels::{Pixels, SurfaceTexture};
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
struct Config {
    interval: Option<u64>,
    duration: Option<u64>,
    flash_interval: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            interval: Some(1200),
            duration: Some(60),
            flash_interval: Some(1000),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(name = "护眼提醒")]
#[command(
    author = "vinciyan",
    version = "v0.1.0",
    about = "每隔一段时间强制闪烁屏幕提醒保护眼睛",
    long_about = "
================================================================================\n\
Overview\n\
================================================================================\n\
每隔一段时间强制闪烁屏幕提醒保护眼睛\n\
================================================================================\n\n\
# Examples\n\n\
## 配置文件\n\n\
config.toml\n\n\
```\n\
interval=60\n\
duration=10\n\
flash_interval=1000\n\
```\n\n\
配置文件可以和程序同一个目录，也可以通过参数`-c`指定配置文件的绝对路径\n\n\
## 控制台参数\n\n\
自定义参数:\n\n\
```sh\n\
eye_care_rs.exe -i 60 -d 10 -f 800
```
或者\n\n\
```sh\n\
eye_care_rs.exe --interval 60 --duration 10 --flash-interval 800
```"
)]
struct Opt {
    /// config.toml文件路径
    #[clap(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    /// 提醒间隔（秒）
    #[clap(short, long)]
    interval: Option<u64>,

    /// 提醒持续时间（秒）
    #[clap(short, long)]
    duration: Option<u64>,

    /// 颜色切换间隔（毫秒）
    #[clap(short = 'f', long)]
    flash_interval: Option<u64>,
}
fn read_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let path = path.as_ref();
    if path.exists() {
        let config_text = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        let config: Config = toml::from_str(&config_text)
            .with_context(|| "Failed to parse TOML")?;
        Ok(config)
    } else {
        println!("Warning: Config file {:?} not found. Using default configuration.", path);
        Ok(Config::default())
    }
}

fn main() -> Result<()>{
    // 解析命令行参数
    let args = Opt::parse();

    // 确定配置文件路径
    let config_path = if let Some(path) = args.config {
        path
    } else {
        env::current_dir()?.join("config.toml")
    };

    // 从指定的配置文件读取配置，如果文件不存在则使用默认配置
    let mut config = read_config(&config_path)
        .with_context(|| format!("Failed to load configuration from {:?}", config_path))?;

    // 使用命令行参数覆盖配置
    if let Some(interval) = args.interval {
        config.interval = Some(interval);
    }
    if let Some(duration)= args.duration {
        config.duration = Some(duration);
    }
    if let Some(flash_interval) = args.flash_interval {
        config.flash_interval = Some(flash_interval);
    }

    // 在控制台输出参数值
    println!("Config file: {:?}", config_path);
    println!("当前使用的参数值：");
    println!("提醒间隔（秒）：{}", config.interval.unwrap_or(1200));
    println!("提醒持续时间（秒）：{}", config.duration.unwrap_or(60));
    println!("颜色切换间隔（毫秒）：{}", config.flash_interval.unwrap_or(1000));

    // 定义定时器参数，使用命令行参数的值
    let reminder_interval = Duration::from_secs(config.interval.unwrap_or(1200));
    let reminder_duration = Duration::from_secs(config.duration.unwrap_or(60));
    let switch_interval = Duration::from_millis(config.flash_interval.unwrap_or(1000));

    // https://doodlewind.github.io/learn-wgpu-cn/beginner/tutorial1-window/#%E4%BD%BF%E7%94%A8-rust-%E7%9A%84%E6%96%B0%E7%89%88%E7%89%B9%E6%80%A7%E8%A7%A3%E6%9E%90%E5%99%A8
    // 通过 env_logger::init() 来启用日志是非常重要的。当 wgpu 遇到各类错误时，它都会用一条通用性的消息抛出 panic，并通过日志 crate 来记录真正的错误信息。这意味着如果不添加 env_logger::init()，wgpu 将静默地退出，从而使你非常困惑！
    env_logger::init();

    // 创建事件循环
    let event_loop = EventLoop::new();

    // 创建一个隐藏的窗口，初始为不可见
    let window = WindowBuilder::new()
        .with_title("护眼提醒")
        .with_decorations(false)  // 无边框窗口
        .with_inner_size(LogicalSize::new(800, 600))
        .with_visible(false)
        .build(&event_loop)
        .unwrap();

    let mut last_reminder = Instant::now();
    let mut reminder_start = None;
    let mut last_switch = Instant::now();
    let mut color_index = 0; // 用于循环颜色的索引

    // 定义颜色数组
    let colors = [
        [0xFF, 0xFF, 0xFF, 0xFF], // 白色
        [0x00, 0x00, 0x00, 0xFF], // 黑色
        [0x00, 0xFF, 0x00, 0xFF], // 绿色
        [0x00, 0x00, 0xFF, 0xFF], // 蓝色
        [0xFF, 0xFF, 0x00, 0xFF], // 黄色
        [0xFF, 0xA5, 0x00, 0xFF], // 橙色
        [0x00, 0xFF, 0xFF, 0xFF], // 青色
        [0xFF, 0x00, 0x00, 0xFF], // 红色
    ];

    // 将 pixels 声明为可选的
    let mut pixels: Option<Pixels> = None;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::MainEventsCleared => {
                let now = Instant::now();

                // 检查是否需要显示提醒
                if reminder_start.is_none() && now - last_reminder >= reminder_interval {
                    // 开始提醒
                    reminder_start = Some(now);
                    // 设置窗口为置顶
                    window.set_always_on_top(true);
                    // 进入全屏模式
                    window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                    window.set_visible(true);

                    // 仅在第一次需要显示提醒时初始化 pixels
                    if pixels.is_none() {
                        let window_size = window.inner_size();
                        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
                        pixels = Some(Pixels::new(window_size.width, window_size.height, surface_texture).unwrap());
                    }

                    // 请求立即重绘
                    window.request_redraw();
                }

                // 如果正在提醒
                if let Some(start_time) = reminder_start {
                    // 计算下一次颜色切换时间
                    let next_switch = last_switch + switch_interval;
                    let next_end = start_time + reminder_duration;
                    let now = Instant::now();

                    if now >= next_switch {
                        last_switch = now;
                        // 更新颜色索引
                        color_index = (color_index + 1) % colors.len();
                        // 请求重绘
                        window.request_redraw();
                    }

                    if now >= next_end {
                        // 提醒结束
                        reminder_start = None;
                        last_reminder = now;
                        window.set_visible(false);
                        // 退出全屏模式
                        window.set_fullscreen(None);
                        // 取消置顶
                        window.set_always_on_top(false);

                        // 销毁 pixels 对象
                        // pixels = None;
                    } else {
                        // 设置下一次事件触发时间，节省 CPU
                        let next_event = std::cmp::min(next_switch, next_end);
                        *control_flow = ControlFlow::WaitUntil(next_event);
                    }
                } else {
                    // 设置下一次提醒的事件触发时间
                    let next_reminder = last_reminder + reminder_interval;
                    *control_flow = ControlFlow::WaitUntil(next_reminder);
                }
            }

            Event::RedrawRequested(_) => {
                if let Some(pixels) = &mut pixels {
                    // 执行渲染操作
                    let frame = pixels.get_frame_mut();
                    let color = colors[color_index];
                    for chunk in frame.chunks_exact_mut(4) {
                        chunk.copy_from_slice(&color);
                    }

                    if let Err(e) = pixels.render() {
                        eprintln!("pixels.render() failed: {:?}", e);
                        *control_flow = ControlFlow::Exit;
                    }
                }
            }

            // Event::WindowEvent { event, .. } => {
            //     // 处理窗口事件，例如窗口大小变化
            //     if let Some(pixels) = &mut pixels {
            //         match event {
            //             WindowEvent::Resized(size) => {
            //                 if let Err(e) = pixels.resize_surface(size.width, size.height) {
            //                     eprintln!("Failed to resize surface: {:?}", e);
            //                     *control_flow = ControlFlow::Exit;
            //                 }
            //                 if let Err(e) = pixels.resize_buffer(size.width, size.height) {
            //                     eprintln!("Failed to resize buffer: {:?}", e);
            //                     *control_flow = ControlFlow::Exit;
            //                 }
            //             }
            //             _ => {}
            //         }
            //     }
            // }

            _ => {}
        }
    });
}
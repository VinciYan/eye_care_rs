# eye_care_rs

使用Rust开发，每隔一段时间强制闪烁屏幕提醒保护眼睛

## 特点

- 使用wgpu开发

## 使用

查看帮助

```
eye_care_rs.exe -h
eye_care_rs.exe --help
```

### 配置文件

config.toml

```
interval=60
duration=10
flash_interval=1000
```

配置文件可以和程序同一个目录，也可以通过参数`-c`指定配置文件的绝对路径

### 控制台参数

自定义参数:

```sh
eye_care_rs.exe -i 60 -d 10 -f 800
```
或者

```sh
eye_care_rs.exe --interval 60 --duration 10 --flash-interval 800
```

## 开发

编译

```
cargo build --release
```

## 参考

- [learn-wgpu-cn](https://doodlewind.github.io/learn-wgpu-cn/beginner/tutorial1-window/)
# os-in-rust

# 项目结构

mbr: 关于mbr的代码，生成mbr.bin

loader: mbr加载Loader，生成loader.bin

empty60M.img: 空的镜像文件，用于生成操作系统镜像

# 开始执行

生成镜像文件：

```
make build
```

会在build/目录下找到hd60M.img文件，这个就是操作系统镜像。


启动qemu运行：

```
make run
```

会启动qemu启动操作系统。

#![no_std]
#![no_main]

use user_lib::*;

const SERVER_PORT: u16 = 6666;

#[no_mangle]
fn main() -> i32 {
    println!("tcptest: tcp test start");
    if fork() == 0 {
        server();
    } else {
        client();
    }

    0
}

fn server() -> i32 {
    println!("tcptest: child process start to run server");
    // 创建socket
    let sockfd = socket(AF_INET as usize, SOCK_STREAM as usize, IPPROTO_TCP as usize);
    if sockfd < 0 {
        println!("tcptest: socket creation failed");
        return -1;
    }

    // 设置服务器地址
    let server_addr = SockIpv4::new_ipv4(SERVER_PORT);

    // 绑定socket
    if bind(sockfd as usize, &server_addr as *const _ as usize, core::mem::size_of::<SockIpv4>()) < 0 {
        println!("tcptest: bind failed");
        return -1;
    }

    // 监听socket
    if listen(sockfd as usize, 5) < 0 {
        println!("tcptest: listen failed");
        return -1;
    }

    println!("tcptest: server is listening on 127.0.0.1:{}", SERVER_PORT);

    // 接受客户端连接
    let mut client_addr = SockIpv4::new_ipv4(0);
    let mut addrlen = core::mem::size_of::<SockIpv4>();

    let client_fd = accept(
        sockfd as usize,
        &mut client_addr as *mut _ as usize,
        &mut addrlen as *mut _ as usize,
    );

    if client_fd < 0 {
        println!("tcptest: server accept failed");
        return -1;
    }

    println!("tcptest: server accepted connection");

    // 接收数据
    sleep(1); // 等待客户端发送数据
    let mut buf = [0u8; 1024];
    let n = read(client_fd as usize, &mut buf);
    if n > 0 {
        println!("tcptest: server received: {}", core::str::from_utf8(&buf[..n as usize]).unwrap());
    }

    // 发送响应
    let response = b"Hello from server!";
    write(client_fd as usize, response);

    // 关闭连接
    close(client_fd as usize);
    close(sockfd as usize);

    exit(0);
}

fn client() -> i32 {
    println!("tcptest: parent process start to run client");
    // 等待服务器启动
    sleep(1);

    // 创建socket
    let sockfd = socket(AF_INET as usize, SOCK_STREAM as usize, IPPROTO_TCP as usize);
    if sockfd < 0 {
        println!("tcptest: client socket creation failed");
        return -1;
    }

    // 设置服务器地址
    let server_addr = SockIpv4::new_ipv4(SERVER_PORT);

    // 连接服务器
    if connect(
        sockfd as usize,
        &server_addr as *const _ as usize,
        core::mem::size_of::<SockIpv4>(),
    ) < 0
    {
        println!("tcptest: client connect failed");
        return -1;
    }

    println!("tcptest: client connected to server");

    // 发送数据
    let message = b"Hello from client!";
    write(sockfd as usize, message);

    // 接收响应
    let mut buf = [0u8; 1024];
    sleep(2); // 等待服务器发送响应
    let n = read(sockfd as usize, &mut buf);
    if n > 0 {
        println!("tcptest: client received: {}", core::str::from_utf8(&buf[..n as usize]).unwrap());
    }

    // 关闭连接
    close(sockfd as usize);
    0
}
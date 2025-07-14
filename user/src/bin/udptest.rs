#![no_std]
#![no_main]

use user_lib::*;

const SERVER_PORT: u16 = 6666;

#[no_mangle]
fn main() -> i32 {
    println!("udptest: UDP test start");
    if fork() == 0 {
        server();
    } else {
        client();
    }

    0
}

fn server() -> i32 {
    println!("udptest: child process start to run server");
    // 创建UDP socket
    let sockfd = socket(AF_INET as usize, SOCK_DGRAM as usize, IPPROTO_UDP as usize);
    if sockfd < 0 {
        println!("udptest: socket creation failed");
        return -1;
    }

    // 设置服务器地址
    let server_addr = SockIpv4::new_ipv4(SERVER_PORT);

    // 绑定socket
    if bind(sockfd as usize, &server_addr as *const _ as usize, core::mem::size_of::<SockIpv4>()) < 0 {
        println!("udptest: bind failed");
        return -1;
    }

    println!("udptest: server is listening on 127.0.0.1:{}", SERVER_PORT);

    // 接收数据
    let mut buf = [0u8; 1024];
    let buflen = buf.len();
    let mut client_addr = SockIpv4::new_ipv4(0);
    let addrlen = core::mem::size_of::<SockIpv4>() as u32;

    let n = recvfrom(
        sockfd as usize,
        &mut buf,
        buflen,
        0,
        &mut client_addr as *mut _,
        &addrlen,
    );

    if n > 0 {
        println!("udptest: server received: {}", core::str::from_utf8(&buf[..n as usize]).unwrap());
        
        // 发送响应
        let response = b"Hello from UDP server!";
        sendto(
            sockfd as usize,
            response,
            response.len(),
            0,
            &client_addr,
            core::mem::size_of::<SockIpv4>(),
        );
    }

    // 关闭socket
    close(sockfd as usize);

    exit(0);
}

fn client() -> i32 {
    println!("udptest: parent process start to run client");
    // 等待服务器启动
    sleep(1);

    // 创建UDP socket
    let sockfd = socket(AF_INET as usize, SOCK_DGRAM as usize, IPPROTO_UDP as usize);
    if sockfd < 0 {
        println!("udptest: client socket creation failed");
        return -1;
    }

    // 设置服务器地址
    let server_addr = SockIpv4::new_ipv4(SERVER_PORT);

    // 发送数据
    let message = b"Hello from UDP client!";
    sendto(
        sockfd as usize,
        message,
        message.len(),
        0,
        &server_addr,
        core::mem::size_of::<SockIpv4>(),
    );

    println!("udptest: client sent message to server");

    // 接收响应
    let mut buf = [0u8; 1024];
    let buflen = buf.len();
    let mut server_addr = SockIpv4::new_ipv4(0);
    let addrlen = core::mem::size_of::<SockIpv4>() as u32;

    sleep(1); // 等待服务器响应
    let n = recvfrom(
        sockfd as usize,
        &mut buf,
        buflen,
        0,
        &mut server_addr,
        &addrlen,
    );

    if n > 0 {
        println!("udptest: client received: {}", core::str::from_utf8(&buf[..n as usize]).unwrap());
    }

    // 关闭socket
    close(sockfd as usize);
    0
}
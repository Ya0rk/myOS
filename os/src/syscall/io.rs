// this file is used for ppoll and pselect6

use core::ptr::NonNull;

use crate::utils::SysResult;

/// wait for some event on a file descriptor
/// ppoll可以选择使用的信号屏蔽字。
/// 若sigmask为空，那么在与信号有关的方面，ppoll的运行状况和poll相同。
/// 否则，sigmask指向一信号屏蔽字，在调用ppoll时，以原子操作的方式安装该信号屏蔽字。
/// 在返回时恢复以前的信号屏蔽字。
/// fds: 传入传出参数，指向struct pollfd类型数组的首元素，每个数组元素指定一个描述符以及对其关心的状态
/// nfds：指明fds指向的数组元素个数
/// timeout：该参数指定ppoll阻塞等待文件描述符就绪的时间
/// 
/// The field fd contains a file descriptor for an open file.  If this
/// field is negative, then the corresponding events field is ignored
/// and the revents field returns zero.
pub async fn sys_ppoll(

) -> SysResult<usize> {
    todo!()
}
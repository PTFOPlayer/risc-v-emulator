#include <sys/syscall.h>
int main()
{
    char * msg = "hello";
    syscall(SYS_write, 1, msg, sizeof(msg));
}

global _start
_start:
	mov rax, 5
	push rax
	mov rax, 3
	pop rbx
	imul rax, rbx
	mov DWORD [rbp-4], eax	mov rax, 60
	mov rdi, 0
	syscall

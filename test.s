global _start
_start:
	mov rax, 5
	push rax
	mov rax, 4
	push rax
	mov rax, 3
	pop rbx
	imul rax, rbx
	pop rbx
	add rax, rbx
	mov rax, 60
	mov rdi, 0
	syscall

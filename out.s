section .data
section .text
	global main
	global functionA
functionA:
	push rbp
	mov rbp, rsp

	; VARIABLE DECLARATION
	mov rax, 2
	sub rsp, 4
	mov DWORD [rsp], eax

	mov rax, 2
	sub rsp, 8
	push rax
	mov rax, 2
	pop rbx
	add rsp, 8
	add rax, rbx
	leave
	ret
main:
	push rbp
	mov rbp, rsp

	; VARIABLE DECLARATION
	mov rax, 3
	sub rsp, 4
	mov DWORD [rsp], eax

  call functionA
	leave
	ret

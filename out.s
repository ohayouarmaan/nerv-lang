section .data
section .text
	global main
main:
	push rbp
	mov rbp, rsp

	; VARIABLE DECLARATION
	mov rax, 4
	sub rsp, 4
	mov DWORD [rsp], eax

	mov rax, 0
	leave
	ret

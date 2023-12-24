<script lang="ts">
	export let source: string = localStorage.getItem('code') ?? '';
	let editor: HTMLElement;

	export async function reset() {
		source = '';
		localStorage.removeItem('code');
	}

	export function save() {
		localStorage.setItem('code', source);
	}

	export function load() {
		source = localStorage.getItem('code') ?? source;
	}

	function handleKey(e: KeyboardEvent) {
		if (e.key === 'Tab') {
			e.preventDefault();

			const doc = editor.ownerDocument.defaultView;
			var sel = doc?.getSelection();

			if (!sel) return;
			for (let i = 0; i < sel.rangeCount; i++) {
				const range = sel.getRangeAt(i);
				var tabNode = document.createTextNode('\t');
				range?.insertNode(tabNode);
				range?.setStartAfter(tabNode);
				range?.setEndAfter(tabNode);
				sel?.addRange(range);
			}
		}
	}

	function handlePaste(e: ClipboardEvent) {
		e.preventDefault();
		e.stopPropagation();

		const clipboardData = e.clipboardData?.getData('Text');
		if (clipboardData) {
			const doc = editor.ownerDocument.defaultView;
			var sel = doc?.getSelection();

			if (!sel) return;
			for (let i = 0; i < sel.rangeCount; i++) {
				const range = sel.getRangeAt(i);
				var tabNode = document.createTextNode(clipboardData as string);
				range?.insertNode(tabNode);
				range?.setStartAfter(tabNode);
				range?.setEndAfter(tabNode);
				sel?.addRange(range);
			}
		}
	}
</script>

<div class="d-flex text-bg-dark w-100 px-1 pt-2" style="height: 85%;">
	<div
		bind:this={editor}
		contenteditable="true"
		class="code-editor"
		bind:innerText={source}
		placeholder="Insert code here..."
		style="white-space: pre;"
		on:keydown={handleKey}
		on:paste={handlePaste}
		role="textbox"
		tabindex="0"
	></div>
</div>

<style>
	.code-editor {
		font-size: larger;
		border: 0;
		width: 100%;
		height: 100%;
		color: white;
		outline: 0;
		resize: none;
		padding: 0;
		margin: 0;
		font-family: var(--bs-font-monospace);
		font-weight: var(--bs-body-font-weight);
		line-height: var(--bs-body-line-height);
		font-size: 0.975rem;
	}
</style>

<script lang="ts">
	import ace from 'brace';
	import '$lib/focus';
	import 'brace/theme/monokai';
	import { onMount } from 'svelte';

	let editor: HTMLElement;
	let aceEditor: ace.Editor;

	const defaultCode = `let main () = Io.print "Hello World"`;

	export function setSource(source: string) {
		aceEditor.setValue(source);
	}

	export function getSource(): string {
		return aceEditor.getValue();
	}

	onMount(async () => {
		aceEditor = ace.edit(editor);
		aceEditor.getSession().setMode('ace/mode/focus');
		aceEditor.setTheme('ace/theme/monokai');
		load();
	});

	export async function reset() {
		setSource(defaultCode);
		localStorage.removeItem('code');
	}

	export function save() {
		localStorage.setItem('code', getSource());
	}

	export function load() {
		setSource(localStorage.getItem('code') ?? defaultCode);
	}
</script>

<div class="d-flex text-bg-dark w-100" style="height: 85%;">
	<div
		bind:this={editor}
		contenteditable="true"
		class="code-editor"
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
		outline: 0;
		padding: 0;
		margin: 0;
		font-family: var(--bs-font-monospace);
		font-weight: var(--bs-body-font-weight);
		line-height: var(--bs-body-line-height);
		font-size: 0.975rem;
	}
</style>

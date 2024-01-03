<script lang="ts">
	import { afterUpdate } from 'svelte';

	type LogMessage = { message: String; timestamp: Date; isError?: boolean };
	export let output: LogMessage[] = [];
	let outputArea: HTMLDivElement;

	export function log(message: string) {
		output.push({ timestamp: new Date(), message });
		output = output;
		outputArea.scrollTo(0, outputArea.scrollHeight);
	}

	export function logError(message: string) {
		output.push({ timestamp: new Date(), message, isError: true });
		output = output;
		outputArea.scrollTo(0, outputArea.scrollHeight);
	}

	export function clear() {
		output = [];
	}

	afterUpdate(() => {
		console.log('afterUpdate');
		if (output) scrollToBottom(outputArea);
	});

	$: if (output && outputArea) {
		console.log('tick');
		scrollToBottom(outputArea);
	}

	const scrollToBottom = async (node: HTMLElement) => {
		node.scroll({ top: node.scrollHeight, behavior: 'smooth' });
	};
</script>

<div class="d-flex flex-column text-bg-dark w-100" style="height: 30%;">
	<span class="bg-body-tertiary bg-dark" data-bs-theme="dark">
		<span class="mx-1">Output</span>
		<button class="btn btn-danger btn-close mx-2" on:click={clear}></button>
	</span>
	<div bind:this={outputArea} class="output-text-area" contenteditable="false">
		{#each output as line}
			<div class="border-bottom border-secondary w-100 bg-dark" class:text-danger={line.isError}>
				<span class="text-primary">{line.timestamp.toLocaleTimeString()}</span>: {line.message}
			</div>
		{/each}
	</div>
</div>

<style>
	.output-text-area {
		background-color: rgb(84, 86, 90);
		color: white;
		height: 100%;
		overflow: auto;
		white-space: pre;
	}
</style>

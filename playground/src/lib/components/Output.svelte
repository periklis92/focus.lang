<script lang="ts">
	type LogMessage = { message: String; timestamp: Date };
	export let output: LogMessage[] = [];

	export function log(message: string) {
		output.push({ timestamp: new Date(), message });
		output = output;
	}
</script>

<div class="d-flex flex-column text-bg-dark w-100" style="height: 30%;">
	<span class="bg-body-tertiary bg-dark" data-bs-theme="dark"><div class="mx-1">Output</div></span>
	<div class="output-text-area" contenteditable="false">
		{#each output as line}
			<div class="border-bottom border-secondary w-100 bg-dark">
				{line.timestamp.toLocaleTimeString()}: {line.message}
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
	}
</style>

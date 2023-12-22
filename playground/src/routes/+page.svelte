<script lang="ts">
	import Sidebar from '$lib/components/Sidebar.svelte';
	import Nav from '$lib/components/Nav.svelte';
	import CodeEditor from '$lib/components/CodeEditor.svelte';
	import Output from '$lib/components/Output.svelte';
	import { Vm } from '$lib/focus-lang/focus_lang';

	let sidebar: Sidebar;
	let codeEditor: CodeEditor;
	let output: Output;
	let source: string;

	document.addEventListener('keydown', (e) => {
		if (e.ctrlKey && e.key === 's') {
			e.preventDefault();
			e.stopPropagation();
			localStorage.setItem('code', source);
		}
	});

	function run() {
		if (source) {
			try {
				const vm = Vm.new_with_std();
				vm.add_event_listener('log', (data: CustomEvent<string>) => {
					output.log(data.detail);
				});
				vm.execute_from_source(source);
			} catch (error) {
				console.error(error);
			}
		}
	}
</script>

<main
	class="d-flex flex-nowrap"
	style="height: 100vh; max-height: 100vh; overflow-x: auto; overflow-y: hidden;"
>
	<Sidebar
		bind:this={sidebar}
		on:selected={(item) => sidebar.setContent('Hello ' + item.detail)}
		menu={[
			{
				title: 'Tutorials',
				icon: 'bi bi-book',
				items: [{ title: '01. Basics', id: 'tutorial-01' }]
			},

			{
				title: 'Examples',
				icon: 'bi bi-journals',
				items: [{ title: '01. Fibonnachi', id: 'example-01' }]
			}
		]}
	/>

	<div class="d-flex flex-column w-100">
		<Nav on:run={run} on:reset={codeEditor.reset} />
		<CodeEditor bind:this={codeEditor} bind:source />

		<Output bind:this={output} />
	</div>
</main>

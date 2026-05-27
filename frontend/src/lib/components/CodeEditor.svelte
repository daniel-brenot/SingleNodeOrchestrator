<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import type * as Monaco from 'monaco-editor';

  interface Props {
    value: string;
    language: string;
    ariaLabel?: string;
  }

  let { value = $bindable(''), language, ariaLabel = 'Code editor' }: Props = $props();
  let editorElement: HTMLDivElement;
  let editor: Monaco.editor.IStandaloneCodeEditor | null = null;
  let monaco: typeof Monaco | null = null;
  let modelDisposer: Monaco.IDisposable | null = null;

  onMount(async () => {
    monaco = await import('monaco-editor');
    editor = monaco.editor.create(editorElement, {
      value,
      language,
      theme: 'vs-dark',
      automaticLayout: true,
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      fontSize: 14,
      tabSize: 2,
      wordWrap: 'on',
    });

    modelDisposer = editor.onDidChangeModelContent(() => {
      value = editor?.getValue() ?? '';
    });
  });

  $effect(() => {
    if (!editor || editor.getValue() === value) {
      return;
    }

    editor.setValue(value);
  });

  $effect(() => {
    const model = editor?.getModel();

    if (!monaco || !model) {
      return;
    }

    monaco.editor.setModelLanguage(model, language);
  });

  onDestroy(() => {
    modelDisposer?.dispose();
    editor?.dispose();
  });
</script>

<div class="code-editor" bind:this={editorElement} role="textbox" aria-label={ariaLabel}></div>

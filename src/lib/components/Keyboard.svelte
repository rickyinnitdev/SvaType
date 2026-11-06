<script lang="ts">
  import type { LayoutData, Stroke } from '../engine';
  import { messages, digits } from '../i18n/index.svelte';
  import fingers from '../../../content/fingers.json';
  let {
    layout,
    highlight = null,
    showFingers = true,
    heat,
  }: {
    layout: LayoutData;
    highlight?: Stroke | null;
    showFingers?: boolean;
    heat?: Record<string, number>;
  } = $props();
  const colors = Object.fromEntries(
    fingers.fingers.map((f) => [f.id, f.color]),
  );
  let rows = $derived(
    Array.from({ length: 5 }, (_, row) =>
      Object.entries(layout.keys)
        .filter(([, k]) => k.row === row)
        .sort(
          ([a, ka], [b, kb]) =>
            (a === 'Backquote' || a === 'IntlBackslash' ? -1 : ka.scan_code) -
            (b === 'Backquote' || b === 'IntlBackslash' ? -1 : kb.scan_code),
        ),
    ),
  );
  let t = $derived(messages());
  let needsShift = $derived(
    highlight?.layer === 'shift' || highlight?.layer === 'shift_altgr',
  );
  let shiftSide = $derived(
    highlight?.finger.startsWith('L') ? 'ShiftRight' : 'ShiftLeft',
  );
</script>

<section class="keyboard" aria-label={t.keyboard}>
  {#each rows as row, index (index)}
    <div class="key-row" style:--offset={`${index === 4 ? 0 : index * 10}px`}>
      {#if index === 3}<div
          class:highlight={needsShift && shiftSide === 'ShiftLeft'}
          class="key modifier"
        >
          {t.shift}
        </div>{/if}
      {#each row as [code, key] (code)}
        <div
          class="key"
          class:space={code === 'Space'}
          class:highlight={highlight?.code === code}
          class:medium-error={(heat?.[code] ?? 0) > 0.1}
          class:high-error={(heat?.[code] ?? 0) > 0.3}
          style:--finger={colors[key.finger]}
          title={heat
            ? `${code}: ${digits(Math.round((heat[code] ?? 0) * 100))}%`
            : code}
        >
          <span class="latin"
            >{code.startsWith('Key')
              ? code.slice(3)
              : code.startsWith('Digit')
                ? code.slice(5)
                : ''}</span
          >
          <span class="shift-char">{key.shift === ' ' ? '␣' : key.shift}</span>
          <span class="base-char"
            >{code === 'Space'
              ? t.space
              : key.base === '\u200b'
                ? '◌'
                : key.base}</span
          >
          {#if showFingers}<span class="finger">{key.finger}</span>{/if}
          {#if heat && heat[code] !== undefined}<span class="rate"
              >{digits(Math.round(heat[code] * 100))}%</span
            >{/if}
        </div>
      {/each}
      {#if index === 3}<div
          class:highlight={needsShift && shiftSide === 'ShiftRight'}
          class="key modifier"
        >
          {t.shift}
        </div>{/if}
      {#if index === 4}<div
          class="key modifier"
          class:highlight={highlight?.layer === 'altgr' ||
            highlight?.layer === 'shift_altgr'}
        >
          {t.altgr}
        </div>{/if}
    </div>
  {/each}
</section>

<style>
  .keyboard {
    contain: layout paint;
    display: grid;
    gap: 6px;
    padding: 4px 0;
  }
  .key-row {
    display: flex;
    gap: 6px;
    padding-left: var(--offset);
  }
  .key {
    position: relative;
    flex: 1;
    min-width: 0;
    height: 64px;
    border: 1px solid var(--line);
    border-bottom: 3px solid var(--finger, var(--line));
    border-radius: 8px;
    background: var(--surface);
    padding: 10px 6px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .key.highlight {
    outline: 3px solid var(--primary);
    outline-offset: -3px;
    background: var(--surface-soft);
  }
  .key.medium-error {
    background: #fff0c2;
    border-bottom-color: #93660b;
    color: #44330a;
  }
  .key.high-error {
    background: var(--error-soft);
    border-bottom-color: var(--error);
    color: var(--error);
  }
  .rate {
    position: absolute;
    font: 10px sans-serif;
    bottom: 0;
    right: 3px;
  }
  .base-char {
    font-size: 26px;
    line-height: 1.5;
  }
  .shift-char {
    position: absolute;
    right: 6px;
    top: 0;
    font-size: 15px;
  }
  .latin {
    position: absolute;
    left: 5px;
    top: 1px;
    font: 10px sans-serif;
    color: var(--muted);
  }
  .finger {
    position: absolute;
    right: 5px;
    bottom: 1px;
    font: 10px sans-serif;
    color: var(--muted);
  }
  .space {
    max-width: 460px;
    flex: 6;
    height: 48px;
  }
  .space .base-char {
    font-size: 16px;
  }
  .key-row:last-child {
    justify-content: center;
  }
  .modifier {
    flex: 1.4;
    font-size: 14px;
  }
  @media (max-height: 800px) {
    .key {
      height: 48px;
    }
    .space {
      height: 40px;
    }
    .base-char {
      font-size: 24px;
    }
  }
  @media (max-height: 650px) {
    .key {
      height: 40px;
    }
    .space {
      height: 36px;
    }
    .base-char {
      font-size: 22px;
    }
    .shift-char {
      font-size: 12px;
    }
  }
</style>

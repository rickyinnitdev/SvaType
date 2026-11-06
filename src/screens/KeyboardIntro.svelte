<script lang="ts">
  import intro from '../../content/keyboard-intro.json';
  import nida from '../../content/layouts/nida.json';
  import legacy from '../../content/layouts/khmer_legacy.json';
  import { user } from '../lib/stores/profile.svelte';
  import { messages, locale } from '../lib/i18n/index.svelte';
  import Keyboard from '../lib/components/Keyboard.svelte';
  import Hands from '../lib/components/Hands.svelte';
  import { configureActivity, activity } from '../lib/stores/activity.svelte';
  import { navigate } from '../lib/router.svelte';
  import type { LayoutData, Stroke } from '../lib/engine';
  let t = $derived(messages());
  let row = $state(0);
  let at = $state(0);
  let layout = $derived(
    (user.profile?.layout_id === 'khmer_legacy' ? legacy : nida) as LayoutData,
  );
  let code = $derived(intro.rows[row].codes[at]);
  let highlight = $derived<Stroke>({
    code,
    layer: 'base',
    produces: layout.keys[code].base ?? '',
    finger: layout.keys[code].finger,
    cluster_index: 0,
  });
  function practice() {
    configureActivity(
      intro.rows[row].codes.map((c) => layout.keys[c].base ?? '').join(''),
      'drill',
      0,
      'keyboard_intro',
    );
    activity.strict = true;
    activity.findKey = true;
    user.resume = null;
    void navigate('practice');
  }
</script>

<section class="page stack">
  <h1>{t.keyboardIntro}</h1>
  <p>{intro.instruction[locale.language]}</p>
  <div class="row">
    {#each intro.rows as item, i (item.id)}<button
        class:selected={row === i}
        onclick={() => {
          row = i;
          at = 0;
        }}>{item.title[locale.language]}</button
      >{/each}
  </div>
  <Hands active={highlight.finger} /><Keyboard
    {layout}
    {highlight}
    showFingers={true}
  />
  <div class="row">
    <button onclick={() => (at = (at + 1) % intro.rows[row].codes.length)}
      >{t.nextKeyDemo}</button
    ><button class="primary" onclick={practice}>{t.findKey}</button>
  </div>
</section>

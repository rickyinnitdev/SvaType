import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
for (const dir of ['paths', 'lessons', 'images', 'schema'])
  mkdirSync(`content/${dir}`, { recursive: true });
const localized = (km, en) => ({ km, en });
const write = (file, value) =>
  writeFileSync(`content/${file}.json`, JSON.stringify(value, null, 2) + '\n');
const lessonIndex = [];
const g3 = [
  ['sitting', 'ឥរិយាបថអង្គុយ', 'Sit comfortably', 'កកក', ['KeyK']],
  [
    'kb_home_row',
    'ស្គាល់ជួរកណ្ដាល',
    'Meet the home row',
    'ថកថក',
    ['KeyF', 'KeyJ', 'KeyK'],
  ],
  [
    'kb_home_left',
    'ដៃឆ្វេងនៅជួរកណ្ដាល',
    'Left hand home row',
    'ាសដថសដថ',
    ['KeyA', 'KeyS', 'KeyD', 'KeyF'],
  ],
  [
    'kb_home_right',
    'ដៃស្ដាំនៅជួរកណ្ដាល',
    'Right hand home row',
    'កលកល',
    ['KeyJ', 'KeyK', 'KeyL', 'Semicolon'],
  ],
  [
    'kb_left_top',
    'ជួរលើខាងឆ្វេង',
    'Left top row',
    'ឆតរឆតរ',
    ['KeyQ', 'KeyW', 'KeyE', 'KeyR', 'KeyT'],
  ],
  [
    'kb_right_top',
    'ជួរលើខាងស្ដាំ',
    'Right top row',
    'យផយផ',
    ['KeyY', 'KeyU', 'KeyI', 'KeyO', 'KeyP'],
  ],
  [
    'kb_left_bottom',
    'ជួរក្រោមខាងឆ្វេង',
    'Left bottom row',
    'ឋខចវបខច',
    ['KeyZ', 'KeyX', 'KeyC', 'KeyV', 'KeyB'],
  ],
  [
    'kb_right_bottom',
    'ជួរក្រោមខាងស្ដាំ',
    'Right bottom row',
    'នមនម',
    ['KeyN', 'KeyM', 'Comma', 'Period', 'Slash'],
  ],
  [
    'kb_index',
    'ម្រាមចង្អុល',
    'Index fingers reach',
    'ថងហតរយថ',
    ['KeyF', 'KeyG', 'KeyH', 'KeyJ', 'KeyR', 'KeyT', 'KeyY', 'KeyU'],
  ],
  [
    'kb_middle',
    'ម្រាមកណ្ដាល',
    'Middle fingers',
    'ដកចដកច',
    ['KeyD', 'KeyE', 'KeyC', 'KeyK', 'KeyI', 'Comma'],
  ],
  [
    'kb_ring',
    'ម្រាមនាង',
    'Ring fingers',
    'សលសល',
    ['KeyS', 'KeyW', 'KeyX', 'KeyL', 'KeyO', 'Period'],
  ],
  [
    'kb_little',
    'ម្រាមកូន',
    'Little fingers',
    'ឆផឋឆផឋ',
    ['KeyA', 'KeyQ', 'KeyZ', 'Semicolon', 'KeyP', 'Slash'],
  ],
  [
    'consonants_1',
    'ព្យញ្ជនៈដំបូង',
    'First consonants',
    'កខងចឆ',
    ['KeyK', 'KeyX', 'KeyG', 'KeyC', 'KeyQ'],
  ],
  [
    'consonants_2',
    'ព្យញ្ជនៈបន្ថែម',
    'More consonants',
    'ដតថទនបពមយរលសហអ',
    ['KeyD', 'KeyT', 'KeyF', 'KeyN', 'KeyB', 'KeyM'],
  ],
  [
    'vowels_1',
    'ស្រៈ ា ិ ី ុ ូ',
    'First vowel keys',
    'កាកិកីកុកូ',
    ['KeyA', 'KeyI', 'KeyU'],
  ],
  [
    'vowels_2',
    'ស្រៈ េ ែ ៃ ោ ៅ',
    'More vowel keys',
    'កេកែកៃកោកៅ',
    ['KeyE', 'KeyS', 'KeyO'],
  ],
  ['space', 'ដកឃ្លា', 'Make a space', 'ក ខ គ ក ខ គ', ['Space', 'KeyK', 'KeyX']],
  [
    'first_words',
    'ពាក្យខ្លីៗ',
    'Your first words',
    'ដៃ ដី គោ តា ទា',
    ['KeyD', 'KeyA', 'KeyI', 'KeyK', 'KeyO', 'KeyT'],
  ],
  [
    'more_words',
    'ហាត់ពាក្យបន្ថែម',
    'Practise short words',
    'កូន មួយ ពីរ បី បួន',
    ['KeyK', 'KeyN', 'KeyM', 'KeyB'],
  ],
  [
    'g3_accuracy',
    'បញ្ចប់ជំហានដំបូង',
    'Your first accuracy challenge',
    'កូន មាន បាយ ។',
    ['KeyK', 'KeyM', 'KeyB', 'KeyC'],
  ],
];
const g4 = [
  [
    'shift_left',
    'Shift ខាងឆ្វេង',
    'Meet Shift',
    'គ ឃ ជ ទ ព',
    ['KeyK', 'KeyX', 'KeyC', 'KeyT', 'KeyB'],
  ],
  [
    'shift_right',
    'Shift និងដៃស្ដាំ',
    'Shift with both hands',
    'ញ ណ ឡ ធ ឌ',
    ['KeyJ', 'KeyN', 'KeyL', 'KeyF', 'KeyD'],
  ],
  [
    'shift_words',
    'ពាក្យប្រើ Shift',
    'Words with Shift',
    'គោ ឃើញ ជើង ទឹក ពូ',
    ['KeyK', 'KeyX', 'KeyC', 'KeyT', 'KeyB'],
  ],
  [
    'coeng_intro',
    'សញ្ញាជើង ្',
    'Meet coeng',
    'ក្ក ក្ខ ក្ម ក្ន',
    ['KeyJ', 'KeyK', 'KeyX', 'KeyM', 'KeyN'],
  ],
  [
    'coeng_words',
    'ពាក្យមានជើង',
    'Words with subscripts',
    'ខ្មែរ ឆ្កែ ឆ្មា ផ្កា',
    ['KeyJ', 'KeyX', 'KeyQ', 'KeyP'],
  ],
  [
    'coeng_two',
    'ជើងពីរ',
    'Two subscripts',
    'ស្ត្រី ក្សត្រ',
    ['KeyJ', 'KeyS', 'KeyT', 'KeyR'],
  ],
  [
    'coeng_ro',
    'ជើង រ',
    'Meet coeng-ro',
    'ព្រះ ត្រី ជ្រូក',
    ['KeyJ', 'KeyR', 'KeyB', 'KeyT'],
  ],
  [
    'coeng_ro_order',
    'លំដាប់ជើង រ',
    'Coeng-ro order',
    'ស្ត្រី ស្រ្តី',
    ['KeyJ', 'KeyR', 'KeyT', 'KeyS'],
  ],
  [
    'independent_1',
    'ស្រៈពេញតួ',
    'Independent vowels',
    'ឪ ឱ ឯ ឬ ឫ',
    ['BracketRight', 'KeyO', 'KeyR'],
  ],
  [
    'independent_2',
    'ពាក្យស្រៈពេញតួ',
    'Independent vowels in words',
    'ឪពុក ឮ ឭ ឧ ឿ',
    ['BracketRight', 'Backslash', 'BracketLeft'],
  ],
  [
    'sign_bantoc',
    'សញ្ញា ់',
    'Meet bantoc',
    'មាន់ សក់ មាត់',
    ['Quote', 'KeyM', 'KeyS'],
  ],
  [
    'sign_nikahit',
    'សញ្ញា ំ',
    'Meet nikahit',
    'ខ្ញុំ ភ្នំ ចាំ',
    ['KeyM', 'Comma', 'KeyA'],
  ],
  [
    'sign_reahmukh',
    'សញ្ញា ះ',
    'Meet reahmukh',
    'ផ្ទះ សេះ ចុះ',
    ['KeyH', 'Comma', 'KeyV'],
  ],
  [
    'sign_yuukaleapintu',
    'សញ្ញា ៈ',
    'Meet yuukaleapintu',
    'កៈ សៈ',
    ['KeyK', 'KeyS'],
  ],
  [
    'register_1',
    'សញ្ញា ៉',
    'Meet muusikatoan',
    'ញ៉ាំ ម៉ែ',
    ['Quote', 'KeyJ', 'KeyM'],
  ],
  [
    'register_2',
    'សញ្ញា ៊',
    'Meet triisap',
    'ប៊ិច ប៊ូ',
    ['Slash', 'KeyB', 'KeyI'],
  ],
  [
    'multipart_vowels',
    'ស្រៈចម្រុះ',
    'Vowel combinations',
    'កុំ កោះ កេះ កាំ',
    ['Comma', 'Semicolon', 'KeyV', 'KeyA'],
  ],
  [
    'khmer_digits',
    'លេខខ្មែរ',
    'Khmer digits',
    '០ ១ ២ ៣ ៤ ៥ ៦ ៧ ៨ ៩',
    ['Digit1', 'Digit2', 'Digit3', 'Digit4', 'Digit5'],
  ],
  [
    'school_words',
    'ពាក្យនៅសាលា',
    'Words at school',
    'សាលា គ្រូ សិស្ស សៀវភៅ',
    ['KeyS', 'KeyK', 'KeyJ', 'BracketLeft'],
  ],
  [
    'family_words',
    'ពាក្យគ្រួសារ',
    'Family words',
    'ម្ដាយ ឪពុក បង ប្អូន',
    ['KeyM', 'KeyB', 'KeyN'],
  ],
  [
    'g4_sentence',
    'ប្រយោគដំបូង',
    'Your first sentence',
    'សិស្ស អាន សៀវភៅ ។',
    ['KeyS', 'KeyG', 'KeyN'],
  ],
  [
    'g4_accuracy',
    'ភាពត្រឹមត្រូវកម្រិតបន្ទាប់',
    'Your next accuracy challenge',
    'យើង រៀន អក្សរ ខ្មែរ ។',
    ['KeyY', 'KeyR', 'KeyG', 'KeyS'],
  ],
];
const sentenceBanks = [3, 4, 5].flatMap(
  (level) =>
    JSON.parse(readFileSync(`content/sentences/${level}.json`)).sentences,
);
const g5Names = [
  ['punctuation', 'វណ្ណយុត្តិ', 'Punctuation'],
  ['zwsp_awareness', 'ការបំបែកពាក្យ', 'Invisible word breaks'],
  ['sentences_1', 'ប្រយោគខ្លី', 'Short sentences'],
  ['sentences_2', 'ប្រយោគច្បាស់លាស់', 'Clear sentences'],
  ['sentences_3', 'ពាក្យក្នុងប្រយោគ', 'Words in sentences'],
  ['sentences_4', 'ប្រយោគពីរ', 'Two sentences'],
  ['paragraphs_1', 'កថាខណ្ឌដំបូង', 'Your first paragraph'],
  ['paragraphs_2', 'អានមុនវាយ', 'Read before you type'],
  ['paragraphs_3', 'រក្សាចង្វាក់', 'Find your rhythm'],
  ['paragraphs_4', 'ហាត់ដោយស្ងប់', 'Calm practice'],
  ['numbers_dates', 'លេខ និងកាលបរិច្ឆេទ', 'Numbers and dates'],
  ['sentence_edit', 'ពិនិត្យ និងកែតម្រូវ', 'Review and correct'],
  ['common_clusters', 'ក្រុមអក្សរដែលជួបញឹកញាប់', 'Common clusters'],
  ['long_words', 'ពាក្យវែង', 'Longer words'],
  ['mixed_text', 'អត្ថបទចម្រុះ', 'Mixed text'],
  ['g5_accuracy_1', 'ភាពត្រឹមត្រូវ ១', 'Accuracy challenge 1'],
  ['g5_accuracy_2', 'ភាពត្រឹមត្រូវ ២', 'Accuracy challenge 2'],
  ['g5_accuracy_3', 'ភាពត្រឹមត្រូវ ៣', 'Accuracy challenge 3'],
  ['g5_review', 'រំលឹកឡើងវិញ', 'Review your skills'],
  ['g5_finish', 'ដំណើរបន្ទាប់របស់អ្នក', 'Your next chapter'],
];
const g5 = g5Names.map(([id, km, en], i) => [
  id,
  km,
  en,
  id === 'numbers_dates'
    ? 'ថ្ងៃទី ០៦ ខែ ១១ ឆ្នាំ ២០២៦ ។'
    : id === 'punctuation'
      ? '។ ៕ ? !'
      : sentenceBanks[i % sentenceBanks.length],
  ['KeyK', 'KeyS', 'KeyR', 'Space'],
]);
const advanced = Array.from({ length: 12 }, (_, i) => [
  `advanced_${i + 1}`,
  `ហាត់កម្រិតខ្ពស់ ${i + 1}`,
  `Advanced practice ${i + 1}`,
  i === 0
    ? '៛ ១០០០ ៛ ២០០០ ។'
    : sentenceBanks.slice(i % 12, (i % 12) + 3).join(' '),
  ['KeyR', 'KeyJ', 'Space'],
]);
const guidance = localized(
  'អង្គុយឱ្យស្រួល ដាក់ជើងលើដី និងសម្រាកស្មា។ ដាក់ម្រាមដៃលើជួរកណ្ដាល។ មើលគ្រាប់ចុចណែនាំ ហើយវាយយឺតៗដោយត្រឹមត្រូវ។',
  'Sit comfortably with feet on the floor and relaxed shoulders. Rest your fingers on the home row. Follow the highlighted key and practise slowly and accurately.',
);
function lesson(entry, index, group) {
  const [id, km, en, text, focus] = entry;
  const chars = Array.from(text).filter((c) => c !== ' ' && c !== '\u200b');
  const finds = [...new Set(chars)]
    .filter((c) => !['ុ', 'ំ', 'ា', 'ះ'].includes(c))
    .slice(0, 4);
  let explain = guidance;
  if (id.includes('home') || id.startsWith('kb_'))
    explain = localized(
      'ដាក់ម្រាមដៃឆ្វេងលើ A S D F និងម្រាមដៃស្ដាំលើ J K L ;។ ប្រើម្រាមដៃតាមពណ៌ និងស្លាកណែនាំ ហើយត្រឡប់ទៅជួរកណ្ដាលវិញ។',
      'Rest left fingers on A S D F and right fingers on J K L ;. Follow the finger color and label, then return to the home row.',
    );
  if (id.includes('vowel'))
    explain = localized(
      'វាយព្យញ្ជនៈមុន រួចវាយស្រៈ។ ស្រៈអាចបង្ហាញនៅខាងលើ ខាងក្រោម ឬជុំវិញព្យញ្ជនៈ។ អនុវត្តតាមគ្រាប់ចុចណែនាំ។',
      'Type the consonant first, then its vowel. A vowel may appear above, below or around the consonant. Follow the keyboard guide.',
    );
  if (id.startsWith('shift'))
    explain = localized(
      'ចុច Shift ដោយដៃម្ខាង និងចុចគ្រាប់អក្សរដោយដៃម្ខាងទៀត។ លែង Shift នៅពេលត្រឡប់ទៅអក្សរធម្មតា។',
      'Hold Shift with the opposite hand from the letter key. Release Shift before returning to the base layer.',
    );
  if (id.startsWith('coeng'))
    explain = localized(
      'សញ្ញា ្ ប្រែព្យញ្ជនៈបន្ទាប់ទៅជាជើង។ វាយ ្ រួចវាយព្យញ្ជនៈ។ ជើង រ អាចវាយមុន ឬក្រោយជើងមួយទៀតបាន។',
      'Coeng ្ turns the following consonant into a subscript. Type coeng, then the consonant. Coeng-ro is accepted before or after another subscript.',
    );
  if (id === 'space' || id === 'zwsp_awareness')
    explain = localized(
      'ក្ដារចុចនីដា ប្រើ Space សម្រាប់ចន្លោះមើលមិនឃើញ។ ប្រើ Shift + Space សម្រាប់ចន្លោះដែលមើលឃើញ។ ការហាត់អត្ថបទមិនតម្រូវឱ្យវាយចន្លោះមើលមិនឃើញទេ។',
      'On NiDA, Space produces an invisible word break. Use Shift + Space for a visible space. Text practice does not require invisible word breaks.',
    );
  if (id.startsWith('register'))
    explain = localized(
      'សញ្ញា ៉ និង ៊ ស្ថិតនៅលើព្យញ្ជនៈ។ វាយព្យញ្ជនៈ សញ្ញា រួចស្រៈ។ មើលរូបអក្សរនៅក្នុងពាក្យ។',
      'The register shifters ៉ and ៊ belong above the consonant. Type the consonant, shifter, then vowel, and compare the result in the word.',
    );
  if (group === 'g5' || group === 'advanced')
    explain =
      id === 'zwsp_awareness'
        ? explain
        : localized(
            'អានអត្ថបទមុនចាប់ផ្តើម។ ហាត់ជាក្រុមអក្សរ និងរក្សាភាពត្រឹមត្រូវ មុនពេលបង្កើនល្បឿន។ ផ្កាយផ្អែកលើភាពត្រឹមត្រូវ។',
            'Read before starting. Work cluster by cluster and keep your accuracy before increasing speed. Stars depend on accuracy.',
          );
  const steps = [
    {
      type: 'explain',
      text: explain,
      ...(id === 'sitting' ? { image: 'posture.svg' } : {}),
    },
    { type: 'demo', keys: focus },
  ];
  if (finds.length)
    steps.push({ type: 'find_key', items: finds, strict: true });
  if (group === 'g3' && index < 16)
    steps.push({ type: 'type_chars', text, policy: 'stop_on_error' });
  else if (group === 'g4' && index < 18)
    steps.push({
      type: index >= 3 && index <= 7 ? 'type_clusters' : 'type_words',
      items: text.split(' '),
      policy: 'stop_on_error',
    });
  else
    steps.push({
      type: group === 'g3' ? 'type_words' : 'type_sentence',
      ...(group === 'g3' ? { items: text.split(' ') } : { text }),
      policy: 'stop_on_error',
    });
  steps.push({
    type: 'challenge',
    text,
    min_accuracy: group === 'g3' ? 70 : 85,
  });
  const value = {
    id,
    version: 1,
    reviewed: false,
    title: localized(km, en),
    focus_keys: focus,
    steps,
  };
  if (group === 'advanced') value.bonus_kpm = 80;
  write(`lessons/${id}`, value);
  lessonIndex.push({ id, title: value.title, reviewed: false });
  return id;
}
const groups = [
  ['g3', g3],
  ['g4', g4],
  ['g5', g5],
  ['advanced', advanced],
];
const ids = Object.fromEntries(
  groups.map(([group, entries]) => [
    group,
    entries.map((entry, i) => lesson(entry, i, group)),
  ]),
);
const paths = [
  ['grade3', 'ថ្នាក់ទី៣', 'Grade 3', 'g3', 'grade'],
  ['grade4', 'ថ្នាក់ទី៤', 'Grade 4', 'g4', 'grade'],
  ['grade5', 'ថ្នាក់ទី៥', 'Grade 5', 'g5', 'grade'],
  ['beginner', 'អ្នកចាប់ផ្តើម', 'Beginner', 'g3', 'skill'],
  ['elementary', 'កម្រិតបឋម', 'Elementary', 'g4', 'skill'],
  ['intermediate', 'កម្រិតមធ្យម', 'Intermediate', 'g5', 'skill'],
  ['advanced', 'កម្រិតខ្ពស់', 'Advanced', 'advanced', 'skill'],
];
const pathIndex = [];
for (const [id, km, en, group, audience] of paths) {
  const lessons = ids[group];
  const size = Math.ceil(lessons.length / 4);
  const units = Array.from(
    { length: Math.ceil(lessons.length / size) },
    (_, i) => ({
      id: `${id}_u${i + 1}`,
      title: localized(`ជំហាន ${i + 1}`, `Unit ${i + 1}`),
      lessons: lessons.slice(i * size, (i + 1) * size),
    }),
  );
  write(`paths/${id}`, { id, title: localized(km, en), audience, units });
  pathIndex.push({ id, title: localized(km, en), audience });
}
write('index', { paths: pathIndex, lessons: lessonIndex });
writeFileSync(
  'content/images/posture.svg',
  '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 180"><g stroke="#176b62" stroke-width="8" fill="none" stroke-linecap="round" stroke-linejoin="round"><path d="M140 95h145M270 95v65M150 95v65M65 85v45h50v30h30M88 65v44h40l25 47M89 80l47 11"/><circle cx="88" cy="38" r="19" fill="#e9b94c"/><path d="M190 86V43h64v43z" fill="#edf4f2"/></g></svg>\n',
);
console.log(
  `Generated ${lessonIndex.length} unreviewed lessons in ${pathIndex.length} paths.`,
);

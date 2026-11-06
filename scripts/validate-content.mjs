import { readdirSync, readFileSync } from 'node:fs';
import { join, relative } from 'node:path';
const definitions = {
  localized: {
    type: 'object',
    required: ['km', 'en'],
    properties: { km: { type: 'string' }, en: { type: 'string' } },
  },
  target: { type: 'string', minLength: 1 },
};
function validate(value, schema, path) {
  if (schema.$ref) schema = definitions[schema.$ref];
  const type =
    value === null ? 'null' : Array.isArray(value) ? 'array' : typeof value;
  if (
    schema.type &&
    !(Array.isArray(schema.type) ? schema.type : [schema.type]).includes(type)
  )
    throw new Error(`${path}: expected ${schema.type}, got ${type}`);
  if (schema.enum && !schema.enum.includes(value))
    throw new Error(`${path}: unexpected value`);
  if (type === 'string' && schema.minLength && value.length < schema.minLength)
    throw new Error(`${path}: empty string`);
  if (type === 'array') {
    if (schema.minItems && value.length < schema.minItems)
      throw new Error(`${path}: not enough items`);
    if (schema.items)
      value.forEach((v, i) => validate(v, schema.items, `${path}[${i}]`));
  }
  if (type === 'object') {
    for (const key of schema.required ?? [])
      if (!(key in value)) throw new Error(`${path}: missing ${key}`);
    for (const [key, child] of Object.entries(value)) {
      const next = schema.properties?.[key] ?? schema.additionalProperties;
      if (next && typeof next === 'object')
        validate(child, next, `${path}.${key}`);
    }
  }
}
const schemas = readdirSync('content/schema')
  .filter((name) => name.endsWith('.json'))
  .flatMap((name) =>
    JSON.parse(readFileSync(join('content/schema', name), 'utf8')),
  );
let count = 0;
function walk(dir) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const file = join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name !== 'schema') walk(file);
      continue;
    }
    if (!entry.name.endsWith('.json')) continue;
    const name = relative('content', file).replaceAll('\\', '/');
    const schema = schemas.find((s) => new RegExp(s.pattern).test(name));
    if (!schema) throw new Error(`No schema for ${name}`);
    const text = readFileSync(file, 'utf8');
    validate(JSON.parse(text), schema.schema, name);
    for (const word of text.match(/[ក-៿]+/g) ?? [])
      if (!khmerWords.has(word)) khmerWords.set(word, name);
    count++;
  }
}
// One spelling per word: students must not see ម្តាយ in one lesson and ម្ដាយ in another.
const khmerWords = new Map();
walk('content');
const COENG_TA = '្ត';
const COENG_DA = '្ដ';
for (const [word, file] of khmerWords) {
  if (!word.includes(COENG_TA)) continue;
  const other = word.replaceAll(COENG_TA, COENG_DA);
  if (khmerWords.has(other))
    throw new Error(
      `Mixed spelling: "${word}" (${file}) and "${other}" (${khmerWords.get(other)}). Use one.`,
    );
}
// Same finger on either hand must share one colour, otherwise the colour legend
// lies for one hand (e.g. gold meaning left ring AND right index).
const fingerColor = new Map();
for (const f of JSON.parse(readFileSync('content/fingers.json', 'utf8'))
  .fingers) {
  const n = f.id.slice(1);
  if (
    fingerColor.has(n) &&
    fingerColor.get(n).toLowerCase() !== f.color.toLowerCase()
  )
    throw new Error(
      `Finger ${n} has different colours on each hand (${fingerColor.get(n)} vs ${f.color} on ${f.id}).`,
    );
  fingerColor.set(n, f.color);
}
console.log(`Validated ${count} content files.`);

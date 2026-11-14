// Flesch–Kincaid grade level (Kincaid et al. 1975), used by the audit tests to keep astronaut-facing
// text plain. Syllables are counted by vowel groups with the usual silent-e correction; this is the
// standard approximation and is accurate to about one grade on English prose.
export function syllables(word: string): number {
  const w = word.toLowerCase().replace(/[^a-z]/g, "");
  if (!w) return 0;
  if (w.length <= 3) return 1;
  const groups = w.replace(/(?:[^laeiouy]es|ed|[^laeiouy]e)$/, "").replace(/^y/, "").match(/[aeiouy]{1,2}/g);
  return Math.max(1, groups?.length ?? 1);
}

export function grade(text: string): number {
  const sentences = text.split(/[.!?;:]+(?:\s|$)/).filter((s) => /[a-z]/i.test(s));
  const words = text.split(/\s+/).filter((w) => /[a-z]/i.test(w));
  if (!words.length || !sentences.length) return 0;
  const syl = words.reduce((n, w) => n + syllables(w), 0);
  return 0.39 * (words.length / sentences.length) + 11.8 * (syl / words.length) - 15.59;
}

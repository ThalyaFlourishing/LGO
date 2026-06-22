// LotRO Gear Optimizer is a crude tool to search for optimal combinations of gear. See the read-me.txt file for instructions.
// Scrolls, plus the food:
//  . CritR 1413
//  . TMast 8329
//  . TactM 5543

// Vars for max tracking and results
let bestSum = 0, highCrit = 0, highTact = 0, highTmit = 0;
let bestCombo = [], highCritCombo = [], highTactCombo = [], highTmitCombo = [];
let bestStats = { crit: 0, tact: 0, tmit: 0 };
let highCritStats = { crit: 0, tact: 0, tmit: 0 };
let highTactStats = { crit: 0, tact: 0, tmit: 0 };
let highTmitStats = { crit: 0, tact: 0, tmit: 0 };
let permCount = 0;
let lines = [];

/**
 * Recursive: All permutations, never repeating item names or references.
 */
function generateCombinations(depth = 0, currentCombo = []) {
  if (depth === allSlots.length) {
    processCombination(currentCombo);
    return;
  }
  const slot = allSlots[depth];
  for (let i = 0; i < slot.items.length; i++) {
    const candidate = slot.items[i];
    // Don't allow duplicate names or objects
    const nameAlreadyUsed = currentCombo.some(item => item.name === candidate.name);
    const objectAlreadyUsed = currentCombo.includes(candidate);
    if (nameAlreadyUsed || objectAlreadyUsed) continue;
    generateCombinations(depth + 1, [...currentCombo, candidate]);
  }
}

/**
 * Process: Sum, update maxes, push line.
 */
function processCombination(combo) {
  // Always initialize all totals keys so zeros are displayed!
  let totals = { crit: 0, tact: 0, tmit: 0 };
  combo.forEach(piece => {
    piece.stats.forEach(stat => {
      if (totals[stat.key] === undefined) totals[stat.key] = 0;
      totals[stat.key] += stat.value;
    });
  });
  const crit = totals.crit;
  const tact = totals.tact;
  const tmit = totals.tmit;
  const sum = crit + tact + tmit;
  const comboNames = allSlots.map((slot, idx) => `${slot.name} - ${combo[idx].name}\n `);
  if (crit > highCrit) {
    highCrit = crit;
    highCritCombo = comboNames;
    highCritStats = { crit, tact, tmit };
  }
  if (tact > highTact) {
    highTact = tact;
    highTactCombo = comboNames;
    highTactStats = { crit, tact, tmit };
  }
  if (tmit > highTmit) {
    highTmit = tmit;
    highTmitCombo = comboNames;
    highTmitStats = { crit, tact, tmit };
  }
  if (sum > bestSum) {
    bestSum = sum;
    bestCombo = comboNames;
    bestStats = { crit, tact, tmit };
  }

// TO ADD REPORT OF ONLY PERMUTATIONS WHICH EXCEED GIVEN STAT MINIMA (IF YOUR MINIMA ARE TOO LOW, THIS WILL RUN OUT OF MEMORY AT ABOUT 48 LINES) :
  if ((crit >= critMinimum) && (tact >= tactMinimum) && (tmit >= tmitMinimum)) {
    const line = `Stats before scrolls and food:  CritR= ${crit},  TactM= ${tact},  TMitg= ${tmit}. \n   ${comboNames.join(' ')}`;
    lines.push(line);
    permCount++;
  };

// TO ADD REPORT OF *EVERY* PERMUTATION (WILL RUN OUT OF MEMORY AT ABOUT 48 LINES) :
//  const line = `Stats: crit = ${crit}, tact = ${tact}, tmit = ${tmit}. —\n   ${comboNames.join(' ')}`;
//  lines.push(line);
}

// Run the algorithm
generateCombinations();

// Output the results
const outputDiv = document.getElementById('output');
const safeJoin = (arr) => (arr && arr.length) ? arr.join(' ') : 'N/A';

const summaryLines = [
  `\nHighest totals across all permutations with stat breakdowns:`,
  `\nMax sum (sum all tracked stats): ${bestSum} —\n  ${safeJoin(bestCombo)}`,
  `  All Stats: crit = ${bestStats.crit}, tact = ${bestStats.tact}, tmit = ${bestStats.tmit}`,
  `\nMax Critical Rating: ${highCrit} —\n  ${safeJoin(highCritCombo)}`,
  `  All Stats: crit = ${highCritStats.crit}, tact = ${highCritStats.tact}, tmit = ${highCritStats.tmit}`,
  `\nMax Tactical Mastery: ${highTact} —\n  ${safeJoin(highTactCombo)}`,
  `  All Stats: crit = ${highTactStats.crit}, tact = ${highTactStats.tact}, tmit = ${highTactStats.tmit}`,
  `\nMax Tactical Mitigation: ${highTmit} —\n  ${safeJoin(highTmitCombo)}`,
  `  All Stats: crit = ${highTmitStats.crit}, tact = ${highTmitStats.tact}, tmit = ${highTmitStats.tmit}`,
].join('\n ');


if (outputDiv) {
  outputDiv.textContent =
// To only print out the maxima:
//    summaryLines;

// To also print out every permutation:
    summaryLines + '\n\n——————————————————————————————————————————————————————\n——————————————————————————————————————————————————————' +
    '\n\n ' + 
    'A total of ' + 
    permCount + 
    ' permutations of gear meet the given targets.' +
    '\nBelow is a complete list of those permutations. It may be... long.\n\n ' + 
    lines.join('\n \n ');
}

// Also log summary to console
// console.log(summaryLines);
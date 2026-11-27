# Frontend Design Context

Last updated: 2026-11-27

## Product state

The frontend is an in-development operator interface with two tabs. The Advisor tab takes the full table (seats, stacks, bets, button, blinds, cards) and shows what to do. The Equity calculator tab takes a hand and a board and estimates equity. It is not a finished consumer product. The UI must communicate this without adding persistent warning banners or visual noise.

All user-facing application copy is written in English.

## Design direction

The interface should remain minimal, dark, and functional. Its character comes from restrained poker references rather than casino decoration.

### Visual principles

- Use near-black graphite surfaces for the application shell and panels.
- Use muted felt green for the table, focus states, and the primary action.
- Keep amber, bright red, and saturated casino colors out of large surfaces.
- Prefer subtle borders and tonal separation over heavy shadows.
- Use generous spacing, short labels, and clear grouping.
- Keep animation brief and respect `prefers-reduced-motion`.

### Layout hierarchy

The header and the tab bar sit above both tabs. The Equity calculator tab follows this order:

1. **Header:** identity and the optional computer-vision control.
2. **Table:** current hole cards and community cards; this is the visual focus.
3. **Card picker:** a compact rank-and-suit input tied to the active card zone.
4. **Game details:** opponents, pot size, amount to call, and derived required equity.
5. **Analysis:** the equity action and result.

On smaller screens the layout collapses to one column. Card sizes use responsive constraints so a five-card board remains visible without horizontal scrolling.

## Advisor tab

The left column holds the table (board, hero cards, blinds, ante, pot and the New hand and Next street helpers), the six seat editors and the card picker. The right column holds the Get advice button, the advice, the actions recorded this hand and the opponent settings.

The instruction is the largest text on the screen and has one colour per action: muted red for fold, neutral for check, green for call, blue for a bet or raise, and violet for all-in. Medium and low confidence show a notice under the reason; low uses the error tone because the table reading may be wrong. The EV of each action sits in a closed `details` element so it does not compete with the instruction.

The hero seat uses the green focus border, and empty seats are dimmed. Resetting the opponent stats asks for a second click in place instead of a browser dialog, which the Tauri webview does not always show.

## Card system

Cards use an off-white paper surface with a compact corner index and a large lower suit mark. The design intentionally resembles a real deck while remaining easy to scan on a dark background.

The four-color convention is used for legibility:

| Suit | Color |
| --- | --- |
| Hearts | Red |
| Diamonds | Blue |
| Clubs | Green |
| Spades | Charcoal |

Cards already assigned to the hand or board remain disabled in the picker. Hovering or focusing a card reveals its remove affordance.

## Betting inputs

Pot size and amount to call are separate, labeled amount controls. Each provides:

- A short explanation of what the value represents.
- A numeric input for direct entry.
- Minus and plus controls for quick adjustments.
- Non-negative value normalization.

Required equity is derived immediately using:

```text
amount to call / (pot size + amount to call)
```

The interface only presents a call-or-fold comparison when a positive call amount and an equity result are both available.

## Equity visualization

Calculated equity is shown as a radial win-probability chart with a direct win/not-win breakdown. A compact comparison scale places the player's equity against the required-equity threshold derived from the betting inputs. The chart uses native SVG and semantic text so the small data set does not require a charting runtime dependency.

## Background image

The selected poker photograph is currently integrated as a trial background pending visual feedback:

- [Playing cards and casino chips on a green felt table — Vitezslav Vylicil / Pexels](https://www.pexels.com/photo/playing-cards-beside-and-casino-chips-on-green-table-5412324/)
- Source page marks the photograph as free to use.
- Retrieved on 2026-07-31.
- Stored locally as `poker-helper/public/images/poker-table.webp` rather than loaded from a remote URL.
- Optimized to 1920 × 1440 WebP at approximately 87 KB.
- Applied to the application shell with a strong graphite overlay and subtle green light. The photograph contains large playing cards, so it is intentionally not applied directly to the interactive table surface where it would compete with the actual hand.

The interactive table keeps its lightweight CSS felt treatment to preserve card readability. The image treatment can be adjusted or the asset replaced after product feedback.

## Accessibility and interaction notes

- Every icon-only action requires an accessible name.
- Active card destinations use more than color alone: border, background, and picker copy all change.
- Disabled controls remain readable and explain what is needed to continue.
- Keyboard focus uses a visible green outline.
- Native input semantics are retained for number and range fields.

## Relevant files

- `poker-helper/src/presentation/App.tsx`
- `poker-helper/src/presentation/pages/AdvisorPage.tsx`
- `poker-helper/src/presentation/pages/EquityPage.tsx`
- `poker-helper/src/presentation/components/AdvicePanel.tsx`
- `poker-helper/src/presentation/components/SeatEditor.tsx`
- `poker-helper/src/presentation/format.ts`
- `poker-helper/src/domain/tableDraft.ts`
- `poker-helper/src/presentation/components/CardView.tsx`
- `poker-helper/src/presentation/components/CardSelector.tsx`
- `poker-helper/src/presentation/components/AmountInput.tsx`
- `poker-helper/src/index.css`

| case | mode | Blink-WebKit | Blink-librsvg | WebKit-librsvg |
|---|---|---|---|---|
| payoff | embedded-font | 3/19 (edge ≤ 1 px, NCC ≥ 0.826) | 13/19 (edge ≤ 1 px, NCC ≥ 0.487) | 12/19 (edge ≤ 1 px, NCC ≥ 0.493) |
| payoff | viewer-font | 3/19 (edge ≤ 1 px, NCC ≥ 0.844) | 19/19 (edge ≤ 1 px, NCC ≥ 0.947) | 18/19 (edge ≤ 1 px, NCC ≥ 0.899) |
| payoff | outlines | 19/19 (edge ≤ 1 px, NCC ≥ 0.997) | 19/19 (edge ≤ 1 px, NCC ≥ 0.996) | 19/19 (edge ≤ 1 px, NCC ≥ 0.999) |
| curves | embedded-font | 3/22 (edge ≤ 1 px, NCC ≥ 0.822) | 17/22 (edge ≤ 2 px, NCC ≥ 0.519) | 11/22 (edge ≤ 2 px, NCC ≥ 0.522) |
| curves | viewer-font | 3/22 (edge ≤ 1 px, NCC ≥ 0.796) | 22/22 (edge ≤ 1 px, NCC ≥ 0.947) | 20/22 (edge ≤ 1 px, NCC ≥ 0.869) |
| curves | outlines | 22/22 (edge ≤ 1 px, NCC ≥ 0.996) | 22/22 (edge ≤ 1 px, NCC ≥ 0.996) | 22/22 (edge ≤ 0 px, NCC ≥ 0.999) |
| surface | embedded-font | 5/22 (edge ≤ 1 px, NCC ≥ 0.825) | 17/22 (edge ≤ 4 px, NCC ≥ 0.640) | 11/22 (edge ≤ 4 px, NCC ≥ 0.637) |
| surface | viewer-font | 6/22 (edge ≤ 1 px, NCC ≥ 0.848) | 22/22 (edge ≤ 1 px, NCC ≥ 0.976) | 18/22 (edge ≤ 1 px, NCC ≥ 0.890) |
| surface | outlines | 22/22 (edge ≤ 1 px, NCC ≥ 0.996) | 22/22 (edge ≤ 1 px, NCC ≥ 0.997) | 22/22 (edge ≤ 1 px, NCC ≥ 0.999) |
| text | embedded-font | 10/23 (edge ≤ 2 px, NCC ≥ 0.853) | 13/23 (edge ≤ 2 px, NCC ≥ 0.568) | 16/23 (edge ≤ 2 px, NCC ≥ 0.491) |
| text | viewer-font | 8/23 (edge ≤ 1 px, NCC ≥ 0.628) | 22/23 (edge ≤ 2 px, NCC ≥ 0.949) | 20/23 (edge ≤ 2 px, NCC ≥ 0.611) |
| text | outlines | 23/23 (edge ≤ 1 px, NCC ≥ 0.996) | 23/23 (edge ≤ 1 px, NCC ≥ 0.995) | 23/23 (edge ≤ 1 px, NCC ≥ 0.999) |

Portable (every label, every pair): {"embedded-font": false, "viewer-font": false, "outlines": true}
Method discriminates (embedded-font fails in librsvg on every case): True

| case | outlines vs native PNG: Blink | WebKit | librsvg | embedded vs outlines: Blink | WebKit | librsvg |
|---|---|---|---|---|---|---|
| payoff | 19/19 (NCC ≥ 0.982) | 19/19 (NCC ≥ 0.983) | 19/19 (NCC ≥ 0.983) | 0/19 (NCC ≥ 0.406) | 0/19 (NCC ≥ 0.300) | 0/19 (NCC ≥ 0.398) |
| curves | 22/22 (NCC ≥ 0.996) | 22/22 (NCC ≥ 0.997) | 22/22 (NCC ≥ 0.997) | 0/22 (NCC ≥ 0.396) | 0/22 (NCC ≥ 0.391) | 0/22 (NCC ≥ 0.409) |
| surface | 21/22 (NCC ≥ 0.959) | 21/22 (NCC ≥ 0.957) | 21/22 (NCC ≥ 0.963) | 0/22 (NCC ≥ 0.475) | 0/22 (NCC ≥ 0.287) | 0/22 (NCC ≥ 0.532) |
| text | 23/23 (NCC ≥ 0.987) | 23/23 (NCC ≥ 0.988) | 23/23 (NCC ≥ 0.988) | 1/23 (NCC ≥ 0.229) | 0/23 (NCC ≥ 0.178) | 0/23 (NCC ≥ 0.190) |

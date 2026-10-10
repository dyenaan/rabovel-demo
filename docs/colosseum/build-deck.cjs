/* Local research blueprint. Product dependencies are intentionally separate. */
const fs = require('node:fs');
const path = require('node:path');
const PptxGenJS = require('/tmp/rabovel-pitch-tools/node_modules/pptxgenjs');

const OUT = __dirname;
const ROOT = path.resolve(OUT, '../..');
const C = {
  bg: 'F7F9FD', navy: '12233D', muted: '50637F', blue: '4175FF',
  pale: 'EAF0FF', white: 'FFFFFF', line: 'D9E2F0', amber: '8D5A00',
  yellow: 'FFF4D8', green: '147563', mint: 'E3F5EF'
};
const sources = {
  S1: ['Colosseum FAQ', 'https://colosseum.com/hackathon'],
  S3: ['Crypto World’s Fair rules', 'https://colosseum.com/legal/Crypto%20World%27s%20Fair%20Hackathon%20Rules.pdf'],
  S4: ['NGX distribution report', 'https://ngxgroup.com/the-quiet-digital-revolution-reshaping-nigerias-capital-market/'],
  S5: ['NGX Invest', 'https://www.invest.ngxgroup.com/'],
  S6: ['NASD N-DSP announcement', 'https://nasdng.com/news/blockchain-technology/'],
  S7: ['Dinari', 'https://dinari.com/'],
  S8: ['xStocks distribution', 'https://xstocks.com/partner'],
  S9: ['CSCS T+2 announcement', 'https://www.cscs.ng/2025/11/cscs-announces-go-live-of-t2-settlement-cycle-in-capital-market/'],
  S10: ['CSCS settlement explanation', 'https://www.cscs.ng/ngt2/common-questions/'],
  S11: ['SEC capital circular', 'https://sec.gov.ng/for-investors/keep-track-of-circulars/revised-minimum-capital-mc-for-regulated-capital-market-entities/'],
  S14: ['cNGN documentation', 'https://docs.cngn.co/'],
  S15: ['cNGN going live', 'https://docs.cngn.co/going-live'],
  S17: ['NGX broker directory', 'https://ngxgroup.com/ngx-download/status-of-trading-license-holders-2/'],
  S18: ['Crafts full project record', 'https://colosseum.com/projects/explore/crafts-1'],
  S19: ['Credible Finance full project record', 'https://colosseum.com/projects/explore/credible-finance-1']
};
const slides = [
  {
    kind: 'cover', section: 'Company hypothesis', title: 'Broker infrastructure for\ncNGN-funded distribution', seconds: 12,
    lead: 'First customer: a Nigerian broker or issuing intermediary.',
    facts: ['Funding', 'Asset delivery', 'Reconciled operations'],
    prompt: 'Describe the buyer and one job Rabovel helps it complete.',
    notes: 'Founder-selected customer. Business positioning is a working hypothesis. Product is demo-only; no commercial activity established. Replace outline phrasing with your own account of the company.'
  },
  {
    kind: 'flow', section: 'Customer problem', title: 'Which part of the workflow is costly?', seconds: 18,
    lead: 'The customer pain still needs firsthand evidence.',
    cards: [
      ['01', 'Funding', 'When are funds usable?\nWho matches the deposit?'],
      ['02', 'Allocation', 'Who determines entitlement?\nWhat happens to refunds?'],
      ['03', 'Delivery', 'Which record proves ownership?\nWhat can fail?'],
      ['04', 'Reconciliation', 'Which records disagree?\nHow do operators repair them?']
    ],
    prompt: 'Add one observed workflow, its baseline and the person responsible.',
    notes: 'These are customer-discovery questions, not measured pain claims. Separate primary-offer allotment from a sale of already allocated inventory. No invented user quote or savings metric.'
  },
  {
    kind: 'cards', section: 'Insight to test', title: 'Three ingredients; one advantage to prove', seconds: 18,
    lead: 'Test the complete workflow against bank-payment/API automation.',
    cards: [
      ['Digital distribution', 'An existing Nigerian market', 'NGX Invest and broker channels already distribute offers. Digitization is the baseline.'],
      ['cNGN funding', 'A local-currency onchain leg', 'Provider docs describe funding and redemption. Rabovel has no live merchant connection.'],
      ['Paired transfers', 'One onchain transaction', 'Payment and token delivery can share a transaction when funds and inventory are prepared.']
    ],
    prompt: 'Explain the expected benefit, added prefunding cost and how you will measure both.',
    refs: ['S4','S5','S14','S15'],
    notes: 'Market activity does not prove demand for Rabovel. The cNGN and paired-transfer advantage is a hypothesis. Banking, eligibility, allocation and legal ownership are separate boundaries.'
  },
  {
    kind: 'product', section: 'Product and execution', title: 'Inspect the two legs of a demo purchase', seconds: 24,
    lead: 'Implemented in code; a verified recording is still required.',
    facts: ['Investor → broker: cNGN payment', 'Issuer → investor: whole-unit Token-2022 delivery'],
    prompt: 'Insert a verified capture, public transaction ID and before/after balances.',
    notes: 'Source: rabovel-BE/apps/rabovel-api/src/mint_setup.rs, purchase transaction around lines 751–805; docs/settlement-flow.md. Issuer partial signature and investor Phantom signature. Investor fee payer needs SOL. Eligibility preparation is a separate transaction. Zero-decimal mint: whole units. No network purchase was run during this research. Backing, pricing and identity checks are simulated. No custom Rabovel program or active transfer hook. Production ledger and recovery remain missing.'
  },
  {
    kind: 'table', section: 'Competitive position', title: 'Win an operator workflow, not a token category', seconds: 17,
    lead: 'Potential differentiation: cNGN + delivery + reconciled intermediary operations.',
    headers: ['Alternative', 'Existing overlap', 'Evidence Rabovel needs'],
    rows: [
      ['Broker tools / NGX Invest', 'Digital primary distribution', 'Measured integration and operations benefit'],
      ['NASD N-DSP', 'Local tokenized securities infrastructure', 'A specific unmet job or partner path'],
      ['Dinari / xStocks', 'Equities infrastructure and distribution', 'Clear local asset rights and lifecycle'],
      ['Crafts / Credible Finance', 'Equity rights / local-fiat operations', 'Reliable execution and buyer demand']
    ],
    prompt: 'Name the buyer’s current alternative and the outcome that would justify switching.',
    refs: ['S5','S6','S7','S8','S18','S19'],
    notes: 'Direct local alternatives and adjacent analogues are different categories. Crafts: Frontier winner; equity-linked startup fundraising, not Nigerian brokerage. Credible Finance: Cypherpunk stablecoins runner-up, C4 accelerator; remittance infrastructure, not securities permission. Their archived demos had execution limits. No feature scorecard or inferred market leadership.'
  },
  {
    kind: 'market', section: 'Market and business model', title: 'Start with buyer economics', seconds: 18,
    lead: '186 active broker entries are a buyer universe, not qualified demand.',
    cards: [['3 firms','Annual target capacity'],['₦6m / firm','Assumed annual fee'],['₦5bn × 5 bps','Aggregate annual value fee']],
    result: '₦20.5m / year',
    prompt: 'Validate qualified firms, budgets, annual supported value and direct costs.',
    refs: ['S17'],
    notes: 'Arithmetic: 3×₦6m + ₦5bn×0.0005 = ₦20.5m. Hypothetical capacity, not forecast, pipeline or traction. Five bps = 0.05%. Costs, taxes, integration revenue and partner payouts excluded. Active broker directory count does not include all issuing firms or prove eligibility. PDF header October 5 versus filename September 21 discrepancy recorded. Full-universe subscription sensitivity: 186×₦6m = ₦1.116bn/year, not validated TAM. Expansion and repeat use required to substantiate venture-scale opportunity.'
  },
  {
    kind: 'gtm', section: 'Go to market', title: 'Earn one scoped design partner', seconds: 17,
    lead: 'Proposed discovery targets; no completed customer pipeline claimed.',
    cards: [['5','Workflow interviews','Find the costly step and budget owner.'],['2','Process walkthroughs','Compare records, timing and bank APIs.'],['1','Scoped partner proposal','Agree instrument, responsibilities and price test.']],
    prompt: 'Identify who can introduce the team and what a buyer must commit to.',
    notes: 'Qualify ten intermediaries from the directory and genuine team access. Targets shown are planned, not traction. Begin historical/anonymized replay and sandbox work. Measure operator minutes per 100 transactions, exception rate, full timing and cost. A proposed 30% time reduction is an acceptance target to negotiate, not a measured result. No outreach sent.'
  },
  {
    kind: 'team', section: 'Founder and market fit', title: 'Connect experience to the next milestone', seconds: 15,
    lead: 'Founder-reported experience; names and specific contributions still needed.',
    cards: [
      ['Technical founder', 'BNB Chain internship\nNeo Financial junior developer', 'Add one relevant task, build contribution and motivation.'],
      ['Finance teammates', 'Experience in Nigeria’s finance industry', 'Add names, roles, observed workflow and actual customer access.'],
      ['Team execution', 'September 24–25 development history', 'Issuance, funding UI, listing, purchase and portfolio refresh; verify the demo.']
    ],
    prompt: 'Explain each person’s role and the work completed during this event.',
    notes: 'These are user-reported employment experiences, not endorsements or partnerships. Do not imply seniority or regulatory credentials. Repository commit dates are shipping evidence, not traction or proof of eligibility. Disclose pre-September-14 code and third-party composition.'
  },
  {
    kind: 'roadmap', section: 'Traction and viability', title: 'Demo today. Evidence gates next.', seconds: 16,
    lead: 'Current status: no real activity, revenue or commercial traction established.',
    cards: [
      ['Days 1–30','Buyer + operating model','Observed pain, defined rights and a workable partner path.'],
      ['Days 31–60','Durable operations','Ledger, retries, ambiguous outcome recovery and reconciliation.'],
      ['Days 61–90','Scoped commercial test','Price discussion, agreed pilot and authorization before live funds.']
    ],
    prompt: 'Describe what must be proven at each gate before advancing.',
    notes: 'Day ranges are proposed post-submission stages, conditional on prior evidence. No regulatory approval or production launch promised in 90 days. Need cNGN terms, actual custody/register model, partner costs and repeat use. Token controls do not create legal ownership.'
  },
  {
    kind: 'close', section: 'Next milestone', title: 'One instrument. One intermediary.\nOne measurable workflow.', seconds: 10,
    lead: 'The next goal is a scoped design partnership, not an invented traction headline.',
    cards: [['Buyer owner','A person accountable for the operational problem.'],['Operating structure','Rights, funds, asset control and responsibilities agreed.'],['Sandbox benchmark','Matched cost, timing and reconciliation evidence.']],
    prompt: 'State the specific help you need and the evidence you will deliver next.',
    notes: 'Use a founder-owned closing. Do not add a fundraising amount without an actual financing plan. This remains a proposed milestone, not an existing partnership.'
  },
  {
    kind: 'rights', appendix: true, section: 'Appendix · Operating model', title: 'Define what the token represents',
    lead: 'Issuer-native equity, custodied beneficial interest or contractual exposure: unresolved.',
    headers: ['Responsibility', 'Decision still required'],
    rows: [
      ['Instrument + investor rights', 'Issuer, voting/dividends, restrictions, exit and insolvency'],
      ['Underlying assets + register', 'Custodian, backing evidence and authoritative ownership record'],
      ['Client funds + signatures', 'Wallet control, escrow/allotment, approvals and segregation'],
      ['Operating permissions', 'Map actual functions with counsel and authorized partners'],
      ['cNGN connection', 'Provider eligibility, fees, liquidity and live approval']
    ],
    prompt: 'Technology-provider positioning does not determine licensing obligations.',
    refs: ['S11','S14','S15'],
    notes: 'SEC January 2026 circular includes RATOP/DAOP ₦1bn, DAPO/DAI ₦500m, digital-asset exchange/custodian ₦2bn, compliance deadline June 30 2027. Classification of Rabovel unresolved. These are capital requirements, not integration costs. Investor self-signing, server issuer signing and broker payment destination describe demo mechanics, not a completed custody model.'
  },
  {
    kind: 'table', appendix: true, section: 'Appendix · Separate demo', title: 'Show proof in 2 minutes 40 seconds',
    lead: 'Verify a real integrated devnet run before recording.',
    headers: ['Segment', 'Time', 'On-screen evidence'],
    rows: [
      ['Environment + baseline','15s','Devnet, mocks disabled, starting balances'],
      ['Issuer inventory','25s','Prepared mint, inventory and live listing'],
      ['Purchase quote','25s','Whole units, price, cNGN total and fee roles'],
      ['Sign + submit','45s','Investor signature and explorer transaction'],
      ['Result','30s','Both transfers and corresponding balance changes'],
      ['Limits','20s','Simulated inputs; legal and recovery boundaries']
    ],
    prompt: 'A transaction explorer proves onchain effects; it does not prove legal share ownership.',
    refs: ['S1'],
    notes: 'Total 160 seconds; separate from 165-second pitch. Some preparation may be prerecorded but must be disclosed. Do not present fixture screens as functional integration. Eligibility preparation occurs separately. Confirmed commitment, bank timing and legal register updates are different concepts. Record only identifiers actually produced by the verified run.'
  },
  {
    kind: 'economics', appendix: true, section: 'Appendix · Economics', title: 'Capacity scenarios, not forecasts',
    lead: 'Annual software + settled-value fees. All commercial inputs are assumptions.',
    headers: ['Scenario','Firms','Fee / firm','Value / year','Revenue / year'],
    rows: [
      ['Initial target','3','₦6m','₦5bn','₦20.5m'],
      ['Established niche','20','₦6m','₦50bn','₦145m'],
      ['Expanded capacity','60','₦12m','₦500bn','₦970m']
    ],
    prompt: 'Direct costs and margins are unknown. Obtain partner quotes before stating profitability.',
    notes: 'Annual revenue = firms×annual fee + aggregate annual settled value×0.0005 (five basis points). Integration fees, taxes and partner payouts excluded. Contribution = revenue − direct cost. Break-even requires positive contribution and fixed operating costs. Directory universe sensitivity is not qualified SAM. Do not annualize cumulative NGX fundraising or use market capitalization as revenue TAM.'
  },
  {
    kind: 'judging', appendix: true, section: 'Appendix · Judging', title: 'Match each claim to evidence',
    lead: 'No public numerical weights found. Address both FAQ and formal event criteria.',
    headers: ['FAQ factor','Available evidence','Gap to close'],
    rows: [
      ['Founder + market fit','Employment + finance experience','Specific roles and observed problem'],
      ['Insight','Testable workflow hypothesis','Firsthand customer and bank/API comparison'],
      ['Product + execution','Purchase code + shipping history','Verified run, feedback and recovery'],
      ['Market size','Buyer universe + price sensitivity','Qualified buyers, budgets and volume'],
      ['Communication','Customer-first outline','Founder voice and precise asset rights'],
      ['Viability','Partner model + fee scenarios','Permissions, costs and repeat use'],
      ['Traction','Explicit demo-only disclosure','Actual demand and later paid usage']
    ],
    prompt: 'Formal rules also call for functionality, impact, novelty, UX and open-source composition.',
    refs: ['S1','S3'],
    notes: 'This is an evidence checklist, not a numerical grade. Describe Solana/Token-2022/ACL composition accurately, provide repository access and disclose pre-existing code. Winner analogues do not substitute for Rabovel customer evidence.'
  },
  {
    kind: 'sources', appendix: true, section: 'Appendix · Research', title: 'Sources and limits',
    lead: 'Research cut: 7 October 2026. Full source register accompanies this deck.',
    refs: Object.keys(sources),
    prompt: 'Company pages describe their products; they do not independently establish performance.',
    notes: 'Public sources, full Copilot project records and Grid discovery were cross-checked. No customer interviews were conducted, no live network purchase was executed and no external outreach was sent. The NGX report is republished press; broker directory has a filename/header date discrepancy. Forecasts and licenses are not established. Deadline October 12 11:59 p.m. PT / October 13 12:59 a.m. Edmonton; submit earlier on October 12 locally. Event rules specify winners by December 5.'
  }
];

const pptx = new PptxGenJS();
pptx.layout = 'LAYOUT_WIDE';
pptx.author = 'Rabovel · research workspace';
pptx.subject = 'Colosseum evidence and founder presentation blueprint';
pptx.title = 'Rabovel — Colosseum pitch blueprint';
pptx.company = 'Rabovel';
pptx.lang = 'en-GB';
pptx.theme = {
  headFontFace: 'Aptos Display', bodyFontFace: 'Aptos', lang: 'en-GB'
};

const boxes = [];
let currentIndex = 0;
function text(slide, value, x, y, w, h, size = 18, color = C.navy, extras = {}) {
  boxes.push({slide:currentIndex, type:'text', text:String(value), x,y,w,h});
  slide.addText(value, {
    x,y,w,h,fontFace:'Aptos',fontSize:size,color,margin:0,
    breakLine:false,valign:'top',paraSpaceAfter:0,lineSpacingMultiple:1,
    ...extras
  });
}
function rect(slide,x,y,w,h,fill=C.white,border=C.line,extras={}) {
  boxes.push({slide:currentIndex,type:'shape',x,y,w,h});
  slide.addShape(pptx.ShapeType.roundRect,{x,y,w,h,rectRadius:0.1,
    fill:{color:fill},line:{color:border,width:0.75},radius:0.12,...extras});
}
function link(slide, label, url, x, y, w, size=10) {
  text(slide,label,x,y,w,0.2,size,C.muted,{hyperlink:{url},underline:true});
}
function footer(slide, data, index) {
  text(slide,'RABOVEL  /  RESEARCH BLUEPRINT  /  07 OCT 2026',0.55,7.08,8,0.2,9,C.muted);
  const label=data.appendix?'APPENDIX':`${data.seconds}s`;
  text(slide,`${label}  ·  ${String(index+1).padStart(2,'0')}`,11.3,7.06,1.5,0.25,10,C.muted,{align:'right'});
}
function base(data,index) {
  currentIndex=index+1;
  const s=pptx.addSlide();s.background={color:C.bg};
  text(s,data.section.toUpperCase(),0.55,0.32,11.9,0.24,10,C.blue,{bold:true,charSpacing:1.5});
  const multi=data.title.includes('\n');
  text(s,data.title,0.55,multi?0.72:0.8,12.15,multi?1.1:0.8,multi?31:30,C.navy,{bold:true,fontFace:'Aptos Display'});
  text(s,data.lead,0.55,multi?1.94:1.7,12.1,0.36,16,C.muted);
  footer(s,data,index);
  const refs=(data.refs||[]).filter(id=>sources[id]);
  s.addNotes([
    'RESEARCH BLUEPRINT — rewrite in your own voice before submission.',
    data.seconds?`Pitch timing target: ${data.seconds} seconds.`:'Interview appendix; excluded from timed pitch.',
    `Founder prompt: ${data.prompt}`,
    data.notes,
    ...refs.map(id=>`${id}: ${sources[id][0]} — ${sources[id][1]}`)
  ].join('\n\n'));
  return s;
}
function prompt(slide,data,y=6.12) {
  rect(slide,0.55,y,12.2,0.68,C.yellow,C.yellow);
  text(slide,'FOUNDER PROMPT',0.75,y+0.12,1.68,0.18,9,C.amber,{bold:true,charSpacing:0.5});
  text(slide,data.prompt,2.48,y+0.12,10,0.4,14,C.amber);
}
function refs(slide,data,y=6.88) {
  text(slide,'Sources',0.58,y,0.58,0.2,8,C.muted);
  let x=1.2;
  for(const id of data.refs||[]) {
    if(!sources[id])continue;
    const label=id;
    const width=0.42;
    if(x+width>12.75)break;
    link(slide,label,sources[id][1],x,y,width,8);
    x+=width+0.1;
  }
}
function cards(slide, data, {y=2.52,h=3.3,numbered=false}={}) {
  const n=data.cards.length;
  const gap=0.22,w=(12.2-gap*(n-1))/n;
  data.cards.forEach((card,i)=>{
    const x=0.55+i*(w+gap);
    rect(slide,x,y,w,h);
    text(slide,card[0],x+0.22,y+0.23,w-0.44,0.6,numbered?32:19,C.blue,{bold:true});
    text(slide,card[1],x+0.22,y+0.92,w-0.44,1.1,20,C.navy,{bold:true});
    if(card[2])text(slide,card[2],x+0.22,y+2.08,w-0.44,h-2.2,15,C.muted);
  });
}
function table(slide,data,{y=2.45,widths=null,rowH=0.73,font=15}={}) {
  const n=data.headers.length;
  const ws=widths||Array(n).fill(12.2/n);
  let x=0.55;
  for(let col=0;col<n;col++) {
    rect(slide,x,y,ws[col],0.43,C.navy,C.navy);
    text(slide,data.headers[col],x+0.13,y+0.09,ws[col]-0.26,0.27,11,C.white,{bold:true});
    x+=ws[col];
  }
  data.rows.forEach((row,r)=>{
    let rx=0.55;const ry=y+0.43+r*rowH;
    row.forEach((cell,c)=>{
      rect(slide,rx,ry,ws[c],rowH,r%2?C.pale:C.white,C.line);
      text(slide,cell,rx+0.13,ry+0.13,ws[c]-0.26,rowH-0.18,font,c===0?C.navy:C.muted,{bold:c===0});
      rx+=ws[c];
    });
  });
}

slides.forEach((d,i)=>{
  const s=base(d,i);
  switch(d.kind) {
    case 'cover': {
      // White background matches the existing unmodified brand image.
      s.background={color:C.white};
      s.addImage({path:path.join(ROOT,'rabovel-FE/public/Rabovel.png'),x:0.55,y:2.38,w:3.45,h:3.45*657/3019});
      rect(s,0.55,3.23,7.74,2.46,C.pale,C.pale);
      text(s,'FUNDING  →  DELIVERY  →  OPERATIONS',0.85,3.57,7.12,0.4,16,C.blue,{bold:true});
      text(s,'Build around the intermediary’s\nmeasurable operational job.',0.85,4.21,7.08,1.08,26,C.navy,{bold:true});
      rect(s,8.62,2.4,4.12,3.29,C.navy,C.navy);
      text(s,'DEMO ONLY',8.94,2.77,3.4,0.32,14,C.white,{bold:true});
      text(s,'No real activity or\ncommercial traction\nestablished.',8.94,3.46,3.4,1.34,24,C.white,{bold:true});
      text(s,'Evidence + hypotheses + founder prompts',8.94,5.04,3.2,0.39,12,'B9C8E4');
      prompt(s,d);break;
    }
    case 'flow':cards(s,d,{numbered:true});prompt(s,d);break;
    case 'cards':cards(s,d);prompt(s,d);refs(s,d);break;
    case 'product': {
      rect(s,0.55,2.5,5.58,2.91,C.navy,C.navy);
      text(s,'ONE SOLANA TRANSACTION',0.8,2.74,5.03,0.28,12,'B9C8E4',{bold:true});
      text(s,d.facts[0],0.8,3.35,5.02,0.62,21,C.white,{bold:true});
      text(s,'+',0.8,4.05,0.3,0.35,24,'B9C8E4');
      text(s,d.facts[1],0.8,4.49,5.02,0.66,20,C.white,{bold:true});
      rect(s,6.39,2.5,6.36,2.91,C.white,C.blue,{line:{color:C.blue,width:1,dash:'dash'}});
      text(s,'ADD VERIFIED PRODUCT CAPTURE',6.71,2.86,5.7,0.44,18,C.blue,{bold:true});
      text(s,'Transaction ID\nStarting + resulting balances\nActual integrated devnet UI',6.71,3.58,5.7,1.31,23,C.muted);
      text(s,'Separate boundaries: eligibility preparation · banking · legal ownership',0.58,5.67,12.1,0.28,14,C.muted);
      prompt(s,d);break;
    }
    case 'table': {
      table(s,d,d.appendix?{rowH:0.48,widths:[2.65,0.8,8.75],font:14}:{rowH:0.77,widths:[3.06,4.14,5],font:16});
      prompt(s,d);refs(s,d);break;
    }
    case 'market': {
      const w=3.92;
      d.cards.forEach((c,j)=>{
        const x=0.55+j*4.14;rect(s,x,2.48,w,1.3);
        text(s,c[0],x+0.2,2.68,w-0.4,0.42,25,C.blue,{bold:true});
        text(s,c[1],x+0.2,3.29,w-0.4,0.28,13,C.muted);
      });
      rect(s,0.55,4.02,12.2,1.47,C.navy,C.navy);
      text(s,d.result,0.82,4.23,5.54,0.6,35,C.white,{bold:true});
      text(s,'HYPOTHETICAL ANNUAL REVENUE\nNot current revenue or a forecast. Costs excluded.',6.68,4.35,5.74,0.74,16,C.white);
      text(s,'Nigeria-only subscriptions have a finite ceiling. Repeat use and expansion remain unproven.',0.58,5.67,12,0.3,14,C.muted);
      prompt(s,d);refs(s,d);break;
    }
    case 'gtm':cards(s,d,{numbered:true});prompt(s,d);break;
    case 'team':cards(s,d);prompt(s,d);break;
    case 'roadmap':cards(s,d);prompt(s,d);break;
    case 'close': {
      d.cards.forEach((c,j)=>{
        const y=2.61+j*0.98;rect(s,0.55,y,12.2,0.81);
        text(s,`0${j+1}`,0.8,y+0.17,0.52,0.37,23,C.blue,{bold:true});
        text(s,c[0],1.54,y+0.2,3,0.34,19,C.navy,{bold:true});
        text(s,c[1],4.7,y+0.22,7.7,0.38,17,C.muted);
      });prompt(s,d);break;
    }
    case 'rights':table(s,d,{y:2.4,widths:[3.3,8.9],rowH:0.64,font:15});prompt(s,d);refs(s,d);break;
    case 'economics': {
      table(s,d,{widths:[3.3,1,2.3,2.4,3.2],rowH:0.65,font:17});
      text(s,'Revenue = firms × annual fee + aggregate annual value × 0.0005',0.62,5.02,12,0.44,19,C.blue,{bold:true});
      text(s,'No cost quotes, margin claim, integration revenue or partner payout assumption.',0.62,5.62,12,0.35,15,C.muted);
      prompt(s,d);break;
    }
    case 'judging':table(s,d,{widths:[2.71,4.42,5.07],rowH:0.43,font:13});prompt(s,d);refs(s,d);break;
    case 'sources': {
      d.refs.forEach((id,j)=>{
        const col=j%2,row=Math.floor(j/2),x=0.65+col*6.2,y=2.44+row*0.4;
        text(s,id,x,y,0.55,0.25,11,C.blue,{bold:true});
        link(s,sources[id][0],sources[id][1],x+0.58,y,5.3,13);
      });
      text(s,'Local evidence: repository inspection + founder report. Customer and legal validation remain pending.',0.62,5.85,12,0.25,12,C.muted);
      prompt(s,d,6.23);break;
    }
  }
});

// Geometry sanity check. This is not an Office rendering test.
for(const b of boxes) {
  if(b.x<0||b.y<0||b.x+b.w>13.334||b.y+b.h>7.5)throw Error(`Out-of-slide box: ${JSON.stringify(b)}`);
}
const total=slides.filter(s=>!s.appendix).reduce((n,s)=>n+s.seconds,0);
if(total!==165)throw Error(`Unexpected pitch duration ${total}`);

function esc(v) {return String(v).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));}
function nl(v) {return esc(v).replace(/\n/g,'<br>');}
function htmlCards(d) {
  return `<div class="cards" style="--count:${d.cards.length}">${d.cards.map(c=>`<article><h3>${nl(c[0])}</h3><h4>${nl(c[1])}</h4>${c[2]?`<p>${nl(c[2])}</p>`:''}</article>`).join('')}</div>`;
}
function htmlTable(d) {
  return `<table><thead><tr>${d.headers.map(h=>`<th>${esc(h)}</th>`).join('')}</tr></thead><tbody>${d.rows.map(r=>`<tr>${r.map(c=>`<td>${esc(c)}</td>`).join('')}</tr>`).join('')}</tbody></table>`;
}
function htmlBody(d) {
  switch(d.kind) {
    case 'cover': {
      const logo=fs.readFileSync(path.join(ROOT,'rabovel-FE/public/Rabovel.png')).toString('base64');
      return `<div class="cover-grid"><div><img class="logo" alt="Rabovel" src="data:image/png;base64,${logo}"><article><h3>FUNDING → DELIVERY → OPERATIONS</h3><h4>Build around the intermediary’s measurable operational job.</h4></article></div><article class="dark"><h3>DEMO ONLY</h3><h4>No real activity or commercial traction established.</h4><p>Evidence + hypotheses + founder prompts</p></article></div>`;
    }
    case 'product':return `<div class="product-grid"><article class="dark"><h3>ONE SOLANA TRANSACTION</h3><h4>${esc(d.facts[0])}</h4><p>+</p><h4>${esc(d.facts[1])}</h4></article><article class="capture"><h3>ADD VERIFIED PRODUCT CAPTURE</h3><p>Transaction ID<br>Starting + resulting balances<br>Actual integrated devnet UI</p></article></div><p class="boundary">Separate boundaries: eligibility preparation · banking · legal ownership</p>`;
    case 'table':case 'rights':case 'judging':return htmlTable(d);
    case 'economics':return htmlTable(d)+`<p class="formula">Revenue = firms × annual fee + aggregate annual value × 0.0005</p><p>Direct costs and margins remain unvalidated.</p>`;
    case 'market':return htmlCards(d)+`<article class="dark result"><h3>${esc(d.result)}</h3><p>Hypothetical annual revenue. Not current revenue or a forecast. Costs excluded.</p></article><p class="boundary">Nigeria-only subscriptions have a finite ceiling. Repeat use and expansion remain unproven.</p>`;
    case 'close':return `<div class="milestones">${d.cards.map((c,j)=>`<article><span>0${j+1}</span><h4>${esc(c[0])}</h4><p>${esc(c[1])}</p></article>`).join('')}</div>`;
    case 'sources':return `<ul class="source-grid">${d.refs.map(id=>`<li><strong>${id}</strong> <a href="${esc(sources[id][1])}" target="_blank" rel="noreferrer">${esc(sources[id][0])}</a></li>`).join('')}</ul>`;
    default:return htmlCards(d);
  }
}
const html=`<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Rabovel · Colosseum pitch blueprint</title>
<style>
:root{color-scheme:light;--navy:#${C.navy};--muted:#${C.muted};--blue:#${C.blue};--bg:#${C.bg};--line:#${C.line}}
*{box-sizing:border-box}body{margin:0;background:#dce3ed;color:var(--navy);font-family:Aptos,Inter,Arial,sans-serif}a{color:var(--blue)}
nav{position:sticky;top:0;z-index:5;background:var(--navy);color:white;padding:12px 24px;display:flex;gap:12px;align-items:center}nav span{flex:1;font-size:14px}button{border:1px solid #617492;background:transparent;color:white;padding:7px 12px;border-radius:6px;cursor:pointer}button:focus-visible,a:focus-visible{outline:3px solid #f6b941;outline-offset:3px}
main{max-width:1280px;margin:28px auto;padding:0 20px}.slide{background:var(--bg);border-radius:8px;min-height:670px;padding:30px 42px 26px;margin:24px 0;box-shadow:0 10px 35px #26385616;display:flex;flex-direction:column;scroll-margin-top:80px}.kicker{color:var(--blue);font-size:12px;letter-spacing:1.3px;text-transform:uppercase;font-weight:700}h1{font-size:35px;line-height:1.14;margin:20px 0 14px;max-width:1100px}p{font-size:17px;line-height:1.4}.lead{color:var(--muted);margin:0 0 24px}.content{flex:1}.cards{display:grid;grid-template-columns:repeat(var(--count),1fr);gap:18px}article{border:1px solid var(--line);background:white;border-radius:12px;padding:22px}h3{color:var(--blue);font-size:20px;margin:0 0 21px}h4{font-size:23px;line-height:1.23;margin:0 0 20px}article p{color:var(--muted);margin:0}.dark{background:var(--navy);border-color:var(--navy);color:white}.dark h3,.dark h4,.dark p{color:white}.cover-grid{display:grid;grid-template-columns:2fr 1fr;gap:24px}.logo{width:290px;max-width:100%;margin:0 0 30px}.product-grid{display:grid;grid-template-columns:1fr 1.14fr;gap:24px}.capture{border:2px dashed var(--blue)}.capture p{font-size:25px;line-height:1.7}.boundary{font-size:15px;color:var(--muted)}.prompt{background:#${C.yellow};color:#${C.amber};padding:16px 18px;margin:28px 0 12px;border-radius:9px;font-size:16px}.prompt strong{font-size:10px;letter-spacing:1px;margin-right:20px}.footer{display:flex;justify-content:space-between;font-size:10px;letter-spacing:.6px;color:var(--muted)}.refs{font-size:10px;margin:0 0 12px;line-height:1.4}.refs a{margin-right:14px;color:var(--muted)}table{border-collapse:collapse;width:100%;font-size:16px}th{background:var(--navy);color:white;text-align:left;font-size:12px;padding:12px}td{padding:14px;border:1px solid var(--line);vertical-align:top}tbody tr:nth-child(even){background:#${C.pale}}td:first-child{font-weight:700}.result{display:flex;align-items:center;gap:40px;margin-top:20px}.result h3{font-size:36px;white-space:nowrap;margin:0}.result p{font-size:16px}.formula{color:var(--blue);font-weight:700}.milestones{display:grid;gap:15px}.milestones article{display:flex;align-items:center;gap:24px}.milestones span{font-size:26px;color:var(--blue);font-weight:700}.milestones h4{width:240px;font-size:20px;margin:0;flex-shrink:0}.source-grid{display:grid;grid-template-columns:1fr 1fr;gap:15px;list-style:none;padding:0;margin:0;font-size:15px}.source-grid strong{color:var(--blue);display:inline-block;width:42px}details{margin:8px 0 0;font-size:13px;color:var(--muted)}details p{font-size:14px;max-width:1050px}summary{cursor:pointer}.appendix table{font-size:14px}.appendix td{padding:11px}
@media(max-width:760px){nav{padding:10px;font-size:12px}nav span{display:none}main{padding:0 10px;margin:14px auto}.slide{padding:24px;min-height:auto}h1{font-size:28px}.cards,.product-grid,.cover-grid,.source-grid{grid-template-columns:1fr}table{font-size:12px}td,th{padding:9px}.result,.milestones article{display:block}.milestones h4{margin:10px 0;width:auto}.result h3{font-size:30px;margin-bottom:12px}.footer{gap:12px}.prompt strong{display:block;margin-bottom:8px}}
@media print{@page{size:13.333in 7.5in;margin:0}body{background:white;-webkit-print-color-adjust:exact;print-color-adjust:exact}nav,details{display:none}main{max-width:none;margin:0;padding:0}.slide{width:13.333in;height:7.5in;min-height:0;border-radius:0;margin:0;padding:.4in .55in;box-shadow:none;break-after:page;overflow:hidden}h1{font-size:30pt;margin:14px 0}.lead{font-size:14pt;margin-bottom:24px}article{padding:18px}h3{font-size:17pt}h4{font-size:20pt}p{font-size:14pt}.prompt{margin-top:20px;padding:12px;font-size:12pt}table{font-size:13pt}.appendix table{font-size:11pt}td{padding:12px}.footer{font-size:8pt}.refs{font-size:7pt}.source-grid{gap:13px;font-size:13pt}}
</style></head><body>
<nav aria-label="Deck controls"><span>RABOVEL · 10 core slides / 2:45 · 5 interview appendices</span><button id="previous" type="button">← Previous</button><button id="next" type="button">Next →</button><button id="print" type="button">Print / PDF</button></nav>
<main>${slides.map((d,i)=>`<section class="slide${d.appendix?' appendix':''}" id="slide-${i+1}" aria-label="Slide ${i+1}: ${esc(d.title.replace(/\n/g,' '))}"><div class="kicker">${esc(d.section)}</div><h1>${nl(d.title)}</h1><p class="lead">${esc(d.lead)}</p><div class="content">${htmlBody(d)}</div><div class="prompt"><strong>FOUNDER PROMPT</strong>${esc(d.prompt)}</div><p class="refs">${(d.refs||[]).map(id=>`<a href="${esc(sources[id][1])}" target="_blank" rel="noreferrer">${esc(id+': '+sources[id][0])}</a>`).join('')}</p><div class="footer"><span>RABOVEL / RESEARCH BLUEPRINT / 07 OCT 2026</span><span>${d.appendix?'APPENDIX':d.seconds+'s'} · ${String(i+1).padStart(2,'0')}</span></div><details><summary>Evidence and speaker notes</summary><p>${nl(d.notes)}</p></details></section>`).join('')}</main>
<script>
const sections=[...document.querySelectorAll('.slide')];
function move(delta){let i=sections.findIndex(s=>s.getBoundingClientRect().bottom>130);i=Math.max(0,Math.min(sections.length-1,i+delta));sections[i].scrollIntoView({behavior:'smooth',block:'start'});}
document.querySelector('#previous').onclick=()=>move(-1);document.querySelector('#next').onclick=()=>move(1);document.querySelector('#print').onclick=()=>window.print();document.addEventListener('keydown',e=>{if(e.target.closest('button,a,summary,input,textarea'))return;if(['ArrowRight','ArrowDown','PageDown'].includes(e.key)){e.preventDefault();move(1);}if(['ArrowLeft','ArrowUp','PageUp'].includes(e.key)){e.preventDefault();move(-1);}});
</script></body></html>`;

async function main() {
  await pptx.writeFile({fileName:path.join(OUT,'rabovel-pitch-blueprint.pptx')});
  fs.writeFileSync(path.join(OUT,'rabovel-pitch-preview.html'),html);
  fs.writeFileSync(path.join(OUT,'deck-content.json'),JSON.stringify({researchDate:'2026-10-07',pitchSeconds:total,sources,slides,geometry:boxes},null,2)+'\n');
  console.log(`Created ${slides.length} editable slides; core pitch ${total}s. PPTX, offline HTML and content manifest written.`);
}
main().catch(e=>{console.error(e);process.exitCode=1;});

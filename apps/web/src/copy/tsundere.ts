import type { Copy } from "./en";

/**
 * ツンデレ日本語コピー。ja.ts と同じ意味を、素直になれない口調で書いたもの。
 * ツンは口調だけ。何をする画面・ボタンなのかは必ず伝わるようにすること。
 */
const tsundere = {
  app: {
    name: "Nullbox",
    subtitle: "ヌルボックス ― べ、別にあんたのためのサポートじゃないんだからね。",
    title: "Nullbox（ヌルボックス）: 困ってるなら聞いてあげなくもないサポート窓口",
    description:
      "Nullbox は怠惰なサポート窓口だから。問題があるっていうなら、いや問題なんかないんだけど、もしあったらチケット専用の秘密リンクを渡してあげるよ。返信は、バックログの獣が見張ってる受信箱から届く・・・べ、別に心配してるわけじゃないんだからね？",
    nav: { submit: "困りごとを言いなさい", shop: "ショップ", inbox: "対応者の受信箱" },
    language: "言語",
    footer: "この受信箱を作るとき、チケットは一枚も傷つけてないわ。当然でしょ。",
  },

  status: {
    open: "来たばっかり",
    in_progress: "み、見てるだけよ",
    waiting_on_customer: "あんたの返事待ち",
    resolved: "片付けてあげた",
    closed: "もう知らない！",
  },

  severity: {
    p1: "全部燃えてるじゃない！",
    p2: "怒りのタイピング中",
    p3: "猫がIT部門に就職したの",
    p4: "ちょっと不便なだけ",
  },

  animals: {
    platypus: "カモノハシ",
    capybara: "カピバラ",
    axolotl: "ウーパールーパー",
    narwhal: "イッカク",
    quokka: "クオッカ",
    pangolin: "センザンコウ",
    tardigrade: "クマムシ",
    red_panda: "レッサーパンダ",
    otter: "カワウソ",
    tanuki: "タヌキ",
    alpaca: "アルパカ",
    sloth: "ナマケモノ。お前のことだよ？",
  },
  anonName: (animal: string, number: number) => `名乗らない${animal} #${number}`,

  ticket: {
    number: (n: number) => `#${n}`,
    notFound: "ないわよ",
    notFoundHint: "チケット #404 はあるの。見つからないだけ。わ、私のせいじゃないからね。",
    unassigned: "まだ誰も",
    severity: "深刻度",
    status: "ステータス",
    assignee: "担当者",
    customer: "差出人",
    opened: (when: string) => `${when}に来た`,
    aging: {
      fresh: "まだ新しい",
      yellowed: "黄ばんできたわよ",
      cobwebs: "クモの巣張ってるじゃない",
      fossil: "化石よ、化石",
    },
  },

  customer: {
    title: "で、何が壊れたのよ？",
    intro: "全部話しなさい。ちゃんと読んであげるから。……そのうちね。たぶん。",
    subject: "件名",
    subjectPlaceholder: "プリンターがこっちを見ている気がするのですが・・・",
    body: "何があったのか言いなさい",
    bodyPlaceholder: "やったこと、エラーメッセージ、そのとき鳴った音とか…",
    severity: "どれくらいヤバいの？",
    name: "名前（言いたくないなら別にいいけど）",
    email: "メールアドレス（教えたくないなら別にいいけど）",
    anonymousHint: "どっちも空欄なら、匿名の名前をつけてあげる。か、感謝しなさいよね。",
    submit: "虚無に送ってあげる",
    submitting: "送ってるわよ、急かさないで…",
    createdTitle: "受け取ってあげたわよ！",
    createdBody: (n: number) =>
      `チケット #${n} は、ちゃんと誰か他の人の問題になったわ。べ、別にあんたのためじゃないけど。`,
    secretLinkTitle: "あんた専用の秘密リンク",
    secretLinkWarning:
      "チケットに戻る方法はこのリンクだけよ。ちゃんとブックマークしなさい。もう一度は送ってあげられないの。だって、あんたが誰なのか知らないんだもの。",
    copyLink: "リンクをコピー",
    copied: "コピーしてあげたわ",
    openTicket: "チケットを開く",
    missingToken:
      "このリンク、秘密の部分が欠けてるじゃない。送信したときに出た、完全なリンクを使いなさい。",
    threadTitle: "やりとり",
    reply: "返信を書く",
    replyPlaceholder: "何か変わった？ 新しい症状は？ まさか、また煙が出たの？",
    send: "返信する",
    closedNotice: "このチケットはもう閉じたの。用があるなら、新しいチケットを作りなさい。",
    sendHint: "Ctrl + Enter でも送れるわよ",
  },

  shop: {
    title: "ショップ",
    intro: "虚無を応援したいの？ 商品はひとつだけ。心を込めて、何も作ってないわ。",
    productName: "Nothing to See Here",
    productDescription:
      "名前のとおり、何もないわよ。即日発送、重さゼロ、絶対に壊れない。……文句ある？",
    price: "1日 ¥100",
    buy: "「無」を定期購入する",
    redirecting: "決済ページに連れていってあげてるところ…",
    paymentHint:
      "支払いは Stripe の決済ページでするの。カード情報はこのサイトには届かないから、安心しなさい。",
    thanksTitle: "あ、ありがと……",
    thanksBody:
      "毎日「無」を届けてあげる。ある意味、もう届いてるけどね。う、嬉しくなんかないんだから。",
    reference: "注文番号",
  },

  worker: {
    inboxTitle: "受信箱",
    filter: "表示",
    filterActive: "まだ生きてるの全部",
    search: "件名で検索",
    searchPlaceholder: "件名で絞り込みなさい…",
    noMatches: "何もないわよ。怪しいわね。",
    back: "受信箱に戻る",
    reply: "お客様に返信",
    internalNote: "社内メモ",
    internalBadge: "社内",
    replyPlaceholder: "安心させる一言を書いてあげなさい…",
    notePlaceholder: "対応者にしか見えないわ。愚痴はほどほどにしなさいよ。",
    send: "送信",
    yeet: "片付ける",
    yeetHint: "チケットを解決済みにするわ。",
    yeeted: "片付けてあげたわよ。べ、別に褒めてほしいわけじゃないんだから。",
    assigneePlaceholder: "名前（空欄で解除）",
    assign: "割り当て",
    assignToMe: "仕方ないわね、やってあげる",
  },

  login: {
    title: "対応者専用よ",
    intro: "受信箱を見たいならログインしなさい。お客様は「困りごとを言いなさい」のほうよ。",
    oneTimeNotice:
      "ログインできるのは、アカウントひとつにつき1回だけよ。ログアウトしたらもう使えないから、次は新しいアカウントを作りなさい。",
    tabLogin: "ログイン",
    tabSignup: "アカウント作成",
    username: "ユーザー名",
    usernameHint:
      "3〜32 文字の英数字・文字と「_」「.」「-」よ。返信にはこの名前が出るんだから、ちゃんと考えなさい。",
    password: "パスワード",
    passwordHint: (min: number) =>
      `${min} 文字以上にしなさい。過去のデータ漏えいで流出したパスワードは使わせないわよ。`,
    signupCode: "登録コード",
    signupCodeHint: "チームリーダーに聞きなさい。私は教えてあげないんだから。",
    submit: "ログイン",
    submitting: "確認してあげてる…",
    signupSubmit: "アカウントを作ってあげる",
    signupSubmitting: "作ってあげてる…",
    created: (username: string) =>
      `アカウント「${username}」を作ってあげたわ。さっさとログインしなさい。`,
    logout: "ログアウト",
    logoutHint: "セッションを終わらせるわ。このアカウントでは二度とログインできないわよ。",
    logoutConfirm:
      "ほ、本当にログアウトするの？このアカウントでは二度とログインできないのよ。次は新しいアカウントが必要になるんだから。",
    signedInAs: (username: string) => `${username}、ま、また来たの？`,
  },

  beast: {
    title: "バックログの獣",
    moods: {
      asleep: "寝てるわよ。起こしたら承知しないんだから。",
      peckish: "お腹が空いてきたみたい。こっちを見てるわよ。",
      grumpy: "機嫌が悪いわね。おやつでもあげたら？",
      kaiju: "怪獣になってるじゃない！さっさと逃げなさい！",
    },
    openCount: (n: string) => `未対応 ${n} 件`,
    oldest: (age: string) => `一番古いの: ${age}`,
  },

  zero: {
    celebration: "受信箱ゼロ……や、やるじゃない。",
    void: "ご褒美の虚無よ。ありがたく味わいなさい。",
  },

  thread: {
    empty: "まだ返信はないわ。……さ、寂しくなんかないわよ。",
    statusChanged: (from: string, to: string) => `ステータス: ${from} → ${to}`,
    itsAlive: "生き返ったわ！お客様の返信で、チケットが墓場から戻ってきたの。",
    severityChanged: (from: string, to: string) => `深刻度: ${from} → ${to}`,
    assigned: (to: string) => `${to} が担当になったわ`,
    unassigned: "担当者なし。誰か拾ってあげなさいよ。",
    system: "Nullbox",
    you: "あんた",
  },

  errors: {
    not_found: "隅々まで探してあげたのよ。ソファの下まで。でも、なかったの。",
    missing_ticket_token: "このページには秘密リンクが必要よ。",
    missing_actor: "先にログインしなさい。",
    forbidden: "ここは通さないわよ。だめなものはだめ。",
    validation_failed: "入力におかしいところがあるわ。各項目を見直しなさい。",
    invalid_status_transition: "そのステータス変更はできないの。時間は前にしか進まないのよ。",
    ticket_closed: "このチケットは完全に閉じてるの。",
    service_unavailable: "サーバーが昼寝してるみたい。少し待ってから、もう一回試しなさい。",
    internal: "こっちで何かが壊れたわ。……ご、ごめん。私の環境では動くんだけど。",
    im_a_teapot: "私はティーポットよ。何か文句ある？",
    unauthenticated: "セッションが切れたわ。もう一回ログインしなさい。",
    invalid_credentials: "ユーザー名かパスワードが違うわよ。",
    account_already_used:
      "このアカウントはもう使用済みよ。ログインは1アカウントにつき1回だけ。新しいアカウントを作りなさい。",
    username_taken: "そのユーザー名はもう使われてるわ。別の名前にしなさい。",
    password_too_short: "パスワードが短すぎるわよ。",
    password_leaked:
      "そのパスワード、過去のデータ漏えいで流出してるじゃない。あ、危ないから別のにしなさいよね。",
    invalid_signup_code: "登録コードが違うわよ。",
    network_error: "サーバーにつながらないわ。コンセント、抜けてるんじゃないの？",
  },

  loading: [
    "データベースに袖の下を渡してあげてるの…",
    "ハムスターに、もっと速く走りなさいって言ってるところ…",
    "ラバーダックに相談中。べ、別に頼ってるわけじゃないわ…",
    "サーバーの機嫌をうかがってあげてる…",
    "再起動すれば直る気がするのよ…",
  ],

  eggs: {
    sudo: "この件、報告するからね。",
    capsLock: "優先度: 大声",
    enterprise: {
      title: "エンタープライズモード™",
      intro: "続ける前に、下の必須項目を全部書きなさい。全部よ。",
      fields: [
        "シナジー整合係数",
        "パラダイムシフト管理番号",
        "決裁者の好きなフォント",
        "FAX番号（必須よ）",
        "メールで済んだはずの会議の回数",
      ],
      submit: "稟議に回す",
      synergizing: "シナジー創出中…",
      exit: "現実に戻る",
    },
  },

  time: {
    unknown: "—",
  },
} satisfies Copy;

export default tsundere;

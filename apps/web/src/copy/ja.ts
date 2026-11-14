import type { Copy } from "./en";

/**
 * 日本語コピー。`satisfies Copy` なので、キーが足りないとコンパイルエラーになる。
 * ジョークは翻訳ではなく日本語で書くこと。
 */
const ja = {
  app: {
    name: "Nullbox",
    subtitle: "ヌルボックス ― サポート、ただし虚無。",
    title: "Nullbox（ヌルボックス）: 困りごとを虚無に報告するサポート窓口",
    description:
      "Nullbox は小さなサポート窓口です。問題を報告すると、チケット専用の秘密リンクが届きます。返信は、バックログの獣が見守る受信箱から届きます。",
    nav: { submit: "問題を報告", shop: "ショップ", inbox: "対応者の受信箱" },
    language: "言語",
    footer: "この受信箱の制作において、チケットは一枚も傷ついておりません。",
  },

  status: {
    open: "生まれたて",
    in_progress: "見つめられ中",
    waiting_on_customer: "ボールはそちらに",
    resolved: "ぶん投げ済み",
    closed: "闇の世界へ送還",
  },

  severity: {
    p1: "全部燃えてる",
    p2: "怒りのタイピング中",
    p3: "猫がIT部門に就職",
    p4: "ちょっと不便",
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
    sloth: "ナマケモノ",
  },
  anonName: (animal: string, number: number) => `匿名の${animal} #${number}`,

  ticket: {
    number: (n: number) => `#${n}`,
    notFound: "見つかりません",
    notFoundHint: "チケット #404 は存在します。ただ、見つからないだけです。お約束。",
    unassigned: "まだ誰も",
    severity: "深刻度",
    status: "ステータス",
    assignee: "担当者",
    customer: "差出人",
    opened: (when: string) => `${when}に作成`,
    aging: {
      fresh: "新鮮",
      yellowed: "黄ばみ始め",
      cobwebs: "クモの巣発生",
      fossil: "もはや化石",
    },
  },

  customer: {
    title: "何が壊れましたか？",
    intro: "全部教えてください。チケットは必ず読みます。いつか。たぶん。",
    subject: "件名",
    subjectPlaceholder: "プリンターがまたこっちを見ている",
    body: "何が起きましたか？",
    bodyPlaceholder: "再現手順、エラーメッセージ、そのとき鳴った音など…",
    severity: "どれくらいヤバい？",
    name: "お名前（任意）",
    email: "メールアドレス（任意）",
    anonymousHint: "どちらも空欄なら、気品ある匿名ネームを差し上げます。",
    submit: "虚無に送信する",
    submitting: "発射中…",
    createdTitle: "チケットを受け付けました！",
    createdBody: (n: number) => `チケット #${n} は正式に誰か他の人の問題になりました。`,
    secretLinkTitle: "あなた専用の秘密リンク",
    secretLinkWarning:
      "このリンクがチケットに戻る唯一の方法です。ブックマークしてください。再送はできません。そもそもあなたが誰なのか知らないので。",
    copyLink: "リンクをコピー",
    copied: "コピーしました！",
    openTicket: "チケットを開く",
    missingToken:
      "このリンクには秘密の部分が欠けています。送信時に表示された完全なリンクを使ってください。",
    threadTitle: "やりとり",
    reply: "返信を追加",
    replyPlaceholder: "進展はありましたか？新しい症状は？新たな煙は？",
    send: "返信する",
    closedNotice: "このチケットは闇の世界へ送還されました。新しいチケットを作成してください。",
    sendHint: "Ctrl + Enter で送信",
  },

  shop: {
    title: "ショップ",
    intro: "虚無を応援しよう。商品はひとつ、心を込めて何も作っていません。",
    productName: "Nothing to See Here",
    productDescription: "名前のとおり、何もありません。即日発送、重さゼロ、絶対に壊れません。",
    price: "1日 ¥100",
    buy: "「無」を定期購入する",
    redirecting: "決済ページへ移動中…",
    paymentHint: "お支払いは Stripe の決済ページで行います。カード情報は当サイトには届きません。",
    thanksTitle: "ありがとうございます！",
    thanksBody: "毎日の「無」をお届けします。ある意味、もう届いています。",
    reference: "注文番号",
  },

  worker: {
    inboxTitle: "受信箱",
    filter: "表示",
    filterActive: "まだ生きているもの全部",
    search: "件名で検索",
    searchPlaceholder: "件名で絞り込み…",
    noMatches: "該当なし。怪しい。",
    back: "受信箱に戻る",
    reply: "お客様に返信",
    internalNote: "社内メモ",
    internalBadge: "社内",
    replyPlaceholder: "安心させる一言をどうぞ…",
    notePlaceholder: "対応者にしか見えません。愚痴はほどほどに。",
    send: "送信",
    yeet: "ぶん投げる",
    yeetHint: "チケットを解決済みにします。",
    yeeted: "太陽に向かってぶん投げました。",
    assigneePlaceholder: "名前（空欄で解除）",
    assign: "割り当て",
    assignToMe: "引き受ける",
  },

  login: {
    title: "対応者専用",
    intro: "受信箱を見るにはログインしてください。お客様は「問題を報告」へどうぞ。",
    oneTimeNotice:
      "各アカウントでログインできるのは1回だけです。ログアウトすると使えなくなるので、次は新しいアカウントを作成してください。",
    tabLogin: "ログイン",
    tabSignup: "アカウント作成",
    username: "ユーザー名",
    usernameHint: "3〜32 文字の英数字・文字、「_」「.」「-」。返信にはこの名前が表示されます。",
    password: "パスワード",
    passwordHint: (min: number) =>
      `${min} 文字以上。過去のデータ漏えいで流出したパスワードは使えません。`,
    signupCode: "登録コード",
    signupCodeHint: "チームリーダーに聞いてください。",
    submit: "ログイン",
    submitting: "確認中…",
    signupSubmit: "アカウントを作成",
    signupSubmitting: "作成中…",
    created: (username: string) =>
      `アカウント「${username}」を作成しました。ログインしてください。`,
    logout: "ログアウト",
    logoutHint: "セッションを終了します。このアカウントでは二度とログインできません。",
    logoutConfirm:
      "ログアウトしますか？このアカウントでは二度とログインできません。次回は新しいアカウントが必要です。",
    signedInAs: (username: string) => `${username} としてログイン中`,
  },

  beast: {
    title: "バックログの獣",
    moods: {
      asleep: "おやすみ中。起こさないで。",
      peckish: "小腹が空いてきた。こちらに気づいている。",
      grumpy: "ご機嫌ななめ。おやつ推奨。",
      kaiju: "怪獣モード。直ちに避難してください。",
    },
    openCount: (n: string) => `未対応 ${n} 件`,
    oldest: (age: string) => `最古: ${age}`,
  },

  zero: {
    celebration: "受信箱ゼロ達成！",
    void: "無料の虚無をお楽しみください。",
  },

  thread: {
    empty: "まだ返信はありません。静寂が耳に痛い。",
    statusChanged: (from: string, to: string) => `ステータス: ${from} → ${to}`,
    itsAlive: "生き返った！お客様の返信でチケットが墓場から蘇りました。",
    severityChanged: (from: string, to: string) => `深刻度: ${from} → ${to}`,
    assigned: (to: string) => `${to} さんが担当になりました`,
    unassigned: "担当者なし。自由の身。",
    system: "Nullbox",
    you: "あなた",
  },

  errors: {
    not_found: "隅々まで探しました。ソファの下まで。ありません。",
    missing_ticket_token: "このページには秘密リンクが必要です。",
    missing_actor: "まずログインしてください。",
    forbidden: "ここを通すわけにはいかない。",
    validation_failed: "入力内容に問題があるようです。各項目をご確認ください。",
    invalid_status_transition: "そのステータス変更はできません。時は一方向にしか流れません。",
    ticket_closed: "このチケットは完全に閉じられています。",
    service_unavailable: "サーバーが昼寝中です。少ししてからもう一度お試しください。",
    internal: "こちら側で何かが壊れました。手元の環境では動くんですけどね。",
    im_a_teapot: "私はティーポットです。",
    unauthenticated: "セッションが終了しました。もう一度ログインしてください。",
    invalid_credentials: "ユーザー名またはパスワードが違います。",
    account_already_used:
      "このアカウントは使用済みです。ログインは1アカウントにつき1回だけです。新しいアカウントを作成してください。",
    username_taken: "そのユーザー名は使われています。別の名前をどうぞ。",
    password_too_short: "パスワードが短すぎます。",
    password_leaked:
      "そのパスワードは過去のデータ漏えいで流出しています。別のものを選んでください。",
    invalid_signup_code: "登録コードが違います。",
    network_error: "サーバーに接続できません。コンセント、抜けてませんか？",
  },

  loading: [
    "データベースに袖の下を渡しています…",
    "ハムスターにもっと速く走るよう交渉中…",
    "ラバーダックに相談中…",
    "サーバーのご機嫌をうかがっています…",
    "再起動すれば直る気がしています…",
  ],

  eggs: {
    sudo: "この件は報告されます。",
    capsLock: "優先度: 大声",
    enterprise: {
      title: "エンタープライズモード™",
      intro: "続行する前に、以下の必須項目をすべてご記入ください。",
      fields: [
        "シナジー整合係数",
        "パラダイムシフト管理番号",
        "決裁者の好きなフォント",
        "FAX番号（必須）",
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

export default ja;

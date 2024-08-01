<h1 align="center">
<img src="./assets/banana.jpg" width="28" />
tg_banana_bot
</h1>
A Self-hosted telegram banana bot.

## What is banana?
**Feel free to tap to earn!**

A telegram mini GameFI project, get your banana now:

- [🍌banana link](https://t.me/OfficialBananaBot/banana?startapp=referral=HHQJ6T4)
- [🍌banana announcement](https://web.telegram.org/a/#-1002230273398)
- Referral code: HHQJ6T4

## Features
- [✔] Auto login.
- [✔] Auto claim banana every 8 hours.
- [✔] Auto tap.
- [✔] Invite.
- [✔] Multi account.

### TODO
- [] Auto compelet tasks.
- [] low memory usage.

## Usage

1. Download app from release page.

2. Grab your `🍌banana` URL from [Telegram Web APP](https://t.me/OfficialBananaBot/banana?startapp=referral=HHQJ6T4).

3. Make sure that `user.json` must be same directory with `tg_banana_bot`.

### user.json
format your `user.json` like this:
```json
{
    "{different account alias name}": {
        "link": "{your link}",
        "access_token": "your access_token, most of time, generated by link when you first login",
        "cookie_token": "your cookie_token, most of time, generated by link when you first login",
        "invite_code": "{input your invite code here}"
    }
}
```

## FAQ
**Q:** How to get your `🍌banana` URL

**A:**
1. Open [Telegram Web APP](https://t.me/OfficialBananaBot/banana?startapp=referral=HHQJ6T4)
2. Click `F12` toggle your devtool open.
3. Click `Play` button left bottom side.
4. Exec code below: `document.querySelector('iframe').src`, then copy it manually.
5. Put it into the `user.json` file, like 
```json
{
    "user1": {
        "link": "{What you paste from step4}"
    }
}
```

## Buy me a coffee

[![BTC](https://img.shields.io/badge/BTC-wallet-F7931A?logo=bitcoin)](https://btcscan.org/ "View BTC address") bc1qfsg983l9adyc6fq96v8qjzax6fk3a39muspers

[![GitHub tag](https://img.shields.io/badge/ETH-wallet-3C3C3D?logo=ethereum)](https://etherscan.io/ "View ETH address") 0x5022519902ffFb8EeA99A8E7Ae53586b879f89Fd0x5022519902ffFb8EeA99A8E7Ae53586b879f89Fd

[![GitHub tag](https://img.shields.io/badge/SOL-wallet-9945FF?logo=solana)](https://solscan.io/ "View SOL address") 6xFaxgz6tr8RV1bvcdfp97apf7usinXJQHtFUUTQ7Sir

## License
GPL-3.0
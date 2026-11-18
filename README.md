# Nova

**English** · [ქართული](#ქართული)

**Nova is a free, open-source, lightweight Minecraft: Java Edition client for Android**, made in Georgia 🇬🇪. It is written in Rust and runs natively on the phone, with no Java virtual machine in between, so it starts fast and plays smoothly.

> Not an official Minecraft product. Not approved by or associated with Mojang or Microsoft.

## Features

- **Native and fast:** Rust and Vulkan, no Java virtual machine and no OpenGL translation.
- **Your own game files:** Nova contains no Minecraft code or assets. You need your own legally obtained copy of Minecraft: Java Edition.
- **Waypoints:** `/wp add NAME`, `/wp list`, `/wp remove NAME`. Save places and see their direction and distance on the screen, so a long trip never loses them (the `waypoints` native mod in [`native-mods/`](native-mods/waypoints)).
- **Touch controls:** a ready-made layout, *Lyen144p V2*, is included (Controls → GoyControl → Presets). Up to 48 custom buttons per layout.
- **Accounts:** Microsoft, Ely.by and offline profiles, managed from the title screen.
- **Mods:** open `.hmod` content mods and `.hnmod` native mods straight from your file manager; the Mods page can compile native mods with GitHub.
- **Controllers and Diagnostics:** controller settings, plus crash reports and performance recordings under Options → Diagnostics.

## Tested on

Poco C65 (arm64, Android). Other phones should work but are not tested yet.

## Download

Get the APK from the repository's **Actions** page: open the latest successful `android` run and download `starling-apk-arm64-v8a` (ARM64 fits most phones; ARMv7, x86 and x86_64 builds are also made). Unzip it, then install the APK. Android may ask you to allow installs from your browser or file manager.

## Using waypoints

| Command | What it does |
|---|---|
| `/wp add NAME` | Saves the place you stand at |
| `/wp list` | Lists the saved places of this world |
| `/wp remove NAME` | Forgets a place |

The five nearest places of the current dimension are shown in the top-left corner, for example `Home ^> 312m` (ahead and to the right, 312 blocks away). To turn it on, open the Mods page and use **Use GitHub** on `waypoints`.

## Build from source

```sh
cargo build --profile fast -p horizon-client                     # desktop build
python3 packaging/android/build_apk.py --preview                 # Android APK (needs the Android SDK/NDK)
```

GitHub Actions builds the Android APKs on every push (the `android` workflow). See [CONTRIBUTING.md](CONTRIBUTING.md) and [`docs/`](docs/README.md).

## License

MIT, see [LICENSE](LICENSE).

---

## ქართული

**Nova არის უფასო, ღია კოდის მქონე და მსუბუქი Minecraft: Java Edition-ის კლიენტი Android-ისთვის**, შექმნილი საქართველოში 🇬🇪. დაწერილია Rust-ზე და მუშაობს პირდაპირ ტელეფონზე, Java-ს ვირტუალური მანქანის გარეშე, ამიტომ სწრაფად იწყება და შეუფერხებლად თამაშობს.

> ეს არ არის Minecraft-ის ოფიციალური პროდუქტი. Mojang-ს და Microsoft-ს მასთან კავშირი არ აქვს და არ დაუმტკიცებიათ.

### შესაძლებლობები

- **სწრაფი და ნატიური:** Rust და Vulkan, Java-ს ვირტუალური მანქანისა და OpenGL-ის გადამყვანის გარეშე.
- **თქვენი საკუთარი თამაშის ფაილები:** Nova-ში Minecraft-ის კოდი ან რესურსები არ შედის. გჭირდებათ თქვენი საკუთარი, კანონიერად შეძენილი Minecraft: Java Edition.
- **გზის წერტილები (Waypoints):** `/wp add სახელი`, `/wp list`, `/wp remove სახელი`. შეინახეთ ადგილები და ეკრანზე ნახეთ მათი მიმართულება და მანძილი, რომ შორს წასვლისას აღარაფერი დაგეკარგოთ (`waypoints` მოდი საქაღალდეში [`native-mods/`](native-mods/waypoints)).
- **სენსორული მართვა:** შედის მზა განლაგება *Lyen144p V2* (Controls → GoyControl → Presets). ერთ განლაგებაში 48 ღილაკამდე.
- **ანგარიშები:** Microsoft, Ely.by და ოფლაინ პროფილები, მართვა სათაურის ეკრანიდან.
- **მოდები:** გახსენით `.hmod` და `.hnmod` ფაილები პირდაპირ ფაილების მენეჯერიდან; ნატიური მოდების დაკომპილება GitHub-ით შეიძლება Mods გვერდიდან.
- **კონტროლერი და Diagnostics:** კონტროლერის პარამეტრები, ავარიების ანგარიშები და წარმადობის ჩანაწერები (Options → Diagnostics).

### გატესტილი მოწყობილობა

Poco C65 (arm64, Android). სხვა ტელეფონებზეც უნდა მუშაობდეს, მაგრამ ჯერ არ არის გატესტილი.

### ჩამოტვირთვა

APK აიღეთ რეპოზიტორიის **Actions** გვერდიდან: გახსენით ბოლო წარმატებული `android` აწყობა და ჩამოტვირთეთ `starling-apk-arm64-v8a` (ARM64 უმეტეს ტელეფონს ერგება; მზადდება ARMv7, x86 და x86_64 ვერსიებიც). გახსენით ZIP და დააინსტალირეთ APK. Android-მა შეიძლება მოგთხოვოთ ბრაუზერიდან ან ფაილების მენეჯერიდან ინსტალაციის დაშვება.

### Waypoints-ის გამოყენება

| ბრძანება | რას აკეთებს |
|---|---|
| `/wp add სახელი` | ინახავს ადგილს, სადაც დგახართ |
| `/wp list` | აჩვენებს ამ სამყაროს შენახულ ადგილებს |
| `/wp remove სახელი` | შლის ადგილს |

მიმდინარე განზომილების ხუთი უახლოესი ადგილი ჩანს ზედა მარცხენა კუთხეში, მაგალითად `სახლი ^> 312m` (წინ და მარჯვნივ, 312 ბლოკში). ჩასართავად გახსენით Mods გვერდი და `waypoints`-ზე დააჭირეთ **Use GitHub**.

### ლიცენზია

MIT, იხილეთ [LICENSE](LICENSE).

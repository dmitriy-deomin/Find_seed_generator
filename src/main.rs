mod color;
mod data;

use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufRead, BufReader, stdout, Write};
use std::path::Path;
use rand::{Rng, thread_rng};
use rand::prelude::SliceRandom;
use rustils::parse::boolean::string_to_bool;
use crate::color::{blue, green, magenta, red};
use bip39::{Language, Mnemonic};
use crate::data::WORDS;

const BACKSPACE: char = 8u8 as char;
const FILE_CONFIG: &str = "confSeedGenerator.txt";


fn main() {
    //Чтение настроек, и если их нет создадим
    //-----------------------------------------------------------------
    let conf = match lines_from_file(&FILE_CONFIG) {
        Ok(text) => { text }
        Err(_) => {
            add_v_file(&FILE_CONFIG, data::get_conf_text());
            lines_from_file(&FILE_CONFIG).unwrap()
        }
    };


    let dlinn_a_pasvord: usize = first_word(&conf[0].to_string()).to_string().parse::<usize>().unwrap();
    let start_perebor = first_word(&conf[1].to_string()).to_string();
    let mode: usize = first_word(&conf[2].to_string()).to_string().parse::<usize>().unwrap();
    let comb_perebor_left_: usize = first_word(&conf[3].to_string()).to_string().parse::<usize>().unwrap();
    let rand_alfabet = string_to_bool(first_word(&conf[4].to_string()).to_string());
    let size_rand_alfabet = first_word(&conf[5].to_string()).to_string().parse::<usize>().unwrap();
    let vivod = string_to_bool(first_word(&conf[6].to_string()).to_string());
    //---------------------------------------------------------------------

    //если укажут меньше или 0
    let comb_perebor_left = if comb_perebor_left_ >= 0 {
        comb_perebor_left_
    } else { 1 };

    //рандом
    let mut rng = rand::thread_rng();


    //-------------------------------------------------------------------------
    // Преобразуем строки в вектор
    let mut lines = WORDS.iter().map(|&s| s.to_string()).collect();
    if rand_alfabet { lines = get_rand_list(lines, size_rand_alfabet) };
    if vivod {
        let version: &str = env!("CARGO_PKG_VERSION");
        println!("{}", blue("==================="));
        println!("{}{}", blue("FIND SEED GENERATOR v:"), magenta(version));
        println!("{}", blue("==================="));

        println!("{}{}", blue("ДЛИНА ФРАЗЫ:"), green(dlinn_a_pasvord));
        if rand_alfabet {
            println!("{}{}", blue("СЛУЧАЙНЫЕ ИЗ СПИСКА:"), green("ВКЛЮЧЕННО"));
            println!("{}{}", blue("-КОЛИЧЕСТВО СЛУЧАЙНЫХ ИЗ СПИСКА:"), green(size_rand_alfabet));
            println!("{}{}", blue("-СПИСОК:"), green(lines.join(" ")));
        };

        println!("{}{}", blue("КОЛИЧЕСТВО СЛОВ ПЕРЕБОРА СЛЕВА:"), green(comb_perebor_left));
        println!("{}{}", blue("НАЧАЛО ПЕРЕБОРА:"), green(start_perebor.clone()));
        //-------------------------------------------------------------------------------
        if mode > 2 {
            println!("{}", red("!!!"));
            println!("{}", red(format!("{mode} ТАКОГО РЕЖИМА РАБОТА ПОКА НЕТ\nесть:\n0 последовательный перебор\n1 рандом\n2 комбинированый")));
            println!("{}", red("!!!"));
            jdem_user_to_close_programm();
        }
        println!("{}{}", blue("РЕЖИМ ГЕНЕРАЦИИ ПАРОЛЯ:"), green(get_mode_text(mode)));

        println!("{}", blue("************************************"));
    }


    //для измерения скорости
    let mut total: u64 = 0;

    let charset_len = lines.len();

    //состовляем начальную позицию
    let mut current_combination = vec![0; dlinn_a_pasvord];

    // Разбиение строки на слова
    let start_perebor_list: Vec<&str> = start_perebor.split(',').collect();
    //заполняем страртовыми значениями для фраз
    for d in comb_perebor_left..dlinn_a_pasvord {
        let position = match start_perebor_list.get(d) {
            Some(&ch) => {
                // Находим позицию слова в charset_chars
                lines.iter().position(|c| c == ch).unwrap_or_else(|| {
                    if vivod { eprintln!("{}", format!("Слово:{} из *начала перебора* не найдено, установлено первое из словаря", ch)); }
                    0
                })
            }
            None => rng.gen_range(0..charset_len),
        };
        current_combination[d] = position;
    }


    //--ГЛАВНЫЙ ЦИКЛ
    // слушаем ответы потоков и если есть шлём новую задачу
    let mut password_string = "инициализация".to_string();
    let mut s = String::new();
    for i in current_combination.iter() {
        s.push_str(lines.get(*i).unwrap());
        s.push(' ');
    }

    password_string = s.trim().to_string();

    let mut info = 0;

    //----------------------------------------------------------------------------------------------
    loop {

        info = info+1;
        if info>1000 {
            if vivod {
                let mut stdout = stdout();
                print!("{}\r{}", BACKSPACE, green(format!("{total} {password_string} + все последнее")));
                stdout.flush().unwrap();
                info = 0;
            } else {
                add_v_file("ТЕКУЩИЙ ПОДБОР.txt", format!("{}\n", password_string));
            }
        }


        //получаем все возможные
        for i in 0..2048 {
            let mut mnemonic_test = String::from(format!("{password_string} "));
            let word_end = data::WORDS[i as usize].to_string();
            mnemonic_test.push_str(&word_end);
            if Mnemonic::validate(&mnemonic_test, Language::English).is_ok() {
                total = total + 1;
                if vivod {
                    add_v_file("mnemonic_list.txt", format!("{}\n", mnemonic_test));
                } else {
                    println!("{}", mnemonic_test);
                }
            }
        }

        // последовательный перебор
        if mode == 0 {
            let mut i = dlinn_a_pasvord;
            while i > 0 {
                i -= 1;
                if current_combination[i] + 1 < charset_len {
                    current_combination[i] += 1;
                    break;
                } else {
                    current_combination[i] = 0;
                }
            }

            if i == 0 && current_combination[0] == charset_len - 1 {
                println!("{}", blue("ГОТОВО,перебраты все возможные"));
                jdem_user_to_close_programm();
                break;
            }

            let mut s = String::new();
            for i in current_combination.iter() {
                s.push_str(lines.get(*i).unwrap());
                s.push(' ');
            }

            password_string = s.trim().to_string();
        }

        //случайный набор строк по длинне
        if mode == 1 {
            let mut k = String::new(); // Создаем пустую строку
            for _ in 0..dlinn_a_pasvord {
                let rand = lines.get(rng.gen_range(0..lines.len())).unwrap();
                k.push_str(rand);
                k.push(' '); // Добавляем разделитель между словами
            }
            k.pop(); // Удаляем последний пробел
            password_string = k;
        }

        //комбенированый режим
        if mode == 2 {
            //будем переберать слева указаное количество
            let mut i = comb_perebor_left;
            while i > 0 {
                i -= 1;
                if current_combination[i] + 1 < charset_len {
                    current_combination[i] += 1;
                    break;
                } else {
                    current_combination[i] = 0;
                }
            }

            if i == 0 && current_combination[0] == charset_len - 1 {
                for f in 0..dlinn_a_pasvord {
                    //заполняем слева начальными значениями
                    if f < comb_perebor_left {
                        current_combination[f] = 0;
                    } else {
                        //остальные рандомно
                        current_combination[f] = rng.gen_range(0..charset_len);
                    }
                }
            }
            let mut s = String::new();
            for i in current_combination.iter() {
                s.push_str(lines.get(*i).unwrap());
                s.push(' ');
            }

            password_string = s.trim().to_string();
        }
    }
}

fn add_v_file(name: &str, data: String) {
    OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(name)
        .expect("cannot open file")
        .write(data.as_bytes())
        .expect("write failed");
}

fn get_mode_text(mode: usize) -> String {
    match mode {
        0 => "ПОСЛЕДОВАТЕЛЬНЫЙ ПЕРЕБОР".to_string(),
        1 => "РАНДОМ".to_string(),
        2 => "КОМБИНИРОВАННЫЙ".to_string(),
        _ => { red("ХЗ").to_string() }
    }
}

fn lines_from_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
    BufReader::new(File::open(filename)?).lines().collect()
}

fn first_word(s: &String) -> &str {
    s.trim().split_whitespace().next().unwrap_or("")
}

fn jdem_user_to_close_programm() {
    // Ожидание ввода пользователя для завершения программы
    println!("{}", blue("Нажмите Enter, чтобы завершить программу..."));
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Ошибка чтения строки");
}

//составляем случайный список из полного
fn get_rand_list(mut list: Vec<String>, size_rand_alfabet: usize) -> Vec<String> {
    let mut rng = thread_rng();
    // Перемешиваем символы
    list.shuffle(&mut rng);

    // Берем первые size_rand_alfabet символов
    let selected_chars: Vec<String> = list.into_iter().take(size_rand_alfabet).collect();

    // Создаем строку из выбранных символов
    selected_chars.into_iter().collect()
}
use eframe::egui;

#[derive(Debug, Clone, PartialEq)]
enum EstadoApp {
    Entrada,
    Validando,
    Buscando,
    Concluido,
}

struct Resultado {
    cpf_formatado: String,
    num_diferencas: usize,
}

struct Aplicativo {
    estado: EstadoApp,
    entrada: String,
    cpf_original: String,
    resultados: Vec<Resultado>,
    mensagem_erro: Option<String>,
    progresso_busca: String,
    total_verificados: usize,
    posicao_scroll: f32,
    logo_texture: Option<egui::TextureHandle>,
}

impl Default for Aplicativo {
    fn default() -> Self {
        Self::novo()
    }
}

impl Aplicativo {
    fn novo() -> Aplicativo {
        Aplicativo {
            estado: EstadoApp::Entrada,
            entrada: String::new(),
            cpf_original: String::new(),
            resultados: Vec::new(),
            mensagem_erro: None,
            progresso_busca: String::new(),
            total_verificados: 0,
            posicao_scroll: 0.0,
            logo_texture: None,
        }
    }

    fn carregar_logo(&mut self, ctx: &egui::Context) {
        if self.logo_texture.is_none() {
            let logo_bytes = include_bytes!("../cpf_quilingue.png");
            let logo_image = image::load_from_memory(logo_bytes).unwrap();
            let tamanho = [logo_image.width() as usize, logo_image.height() as usize];
            let pixels = logo_image.to_rgba8();
            let cor_image = egui::ColorImage::from_rgba_unmultiplied(tamanho, pixels.as_raw());
            
            self.logo_texture = Some(ctx.load_texture(
                "logo",
                cor_image,
                egui::TextureOptions::LINEAR,
            ));
        }
    }

    fn validar_e_buscar(&mut self) {
        self.estado = EstadoApp::Validando;
        self.mensagem_erro = None;
        self.resultados.clear();
        self.total_verificados = 0;
        self.posicao_scroll = 0.0;

        // Remove formatação para validar
        let cpf_limpo = self.entrada.replace(".", "").replace("-", "");

        if cpf_limpo.len() != 11 {
            self.mensagem_erro = Some("CPF deve ter 11 dígitos!".to_string());
            self.estado = EstadoApp::Entrada;
            return;
        }

        if !cpf::valid(&cpf_limpo) {
            self.mensagem_erro = Some("CPF inválido!".to_string());
            self.estado = EstadoApp::Entrada;
            return;
        }

        // CPF válido, iniciar busca
        self.cpf_original = cpf_limpo.clone();
        self.estado = EstadoApp::Buscando;
        self.buscar_variacoes(cpf_limpo);
    }

    fn buscar_variacoes(&mut self, cpf: String) {
        let digitos_cpf: Vec<char> = cpf.chars().collect();
        
        // Tentar variações com 1, 2 e 3 dígitos alterados
        for num_mudancas in 1..=3 {
            self.progresso_busca = format!("Buscando variações com {} dígito(s) alterado(s)...", num_mudancas);
            
            if self.encontrar_variacoes(&digitos_cpf, num_mudancas) {
                // Se encontrou alguma variação com esse número de mudanças, parar
                break;
            }
        }

        self.estado = EstadoApp::Concluido;
        if self.resultados.is_empty() {
            self.progresso_busca = format!(
                "Busca concluída. Nenhuma variação válida encontrada. Total verificado: {}",
                self.total_verificados
            );
        } else {
            self.progresso_busca = format!(
                "Busca concluída! {} variação(ões) encontrada(s). Total verificado: {}",
                self.resultados.len(),
                self.total_verificados
            );
        }
    }

    fn encontrar_variacoes(&mut self, digitos_cpf: &[char], num_mudancas: usize) -> bool {
        let posicoes: Vec<usize> = (0..11).collect();
        let combinacoes = gerar_combinacoes(&posicoes, num_mudancas);
        
        let mut encontrado = false;
        for indices in combinacoes {
            self.tentar_combinacoes_digitos(digitos_cpf, &indices, &mut encontrado);
        }

        encontrado
    }

    fn tentar_combinacoes_digitos(
        &mut self,
        digitos_cpf: &[char],
        posicoes: &[usize],
        encontrado: &mut bool,
    ) {
        // Gerar todas as combinações possíveis de dígitos para as posições especificadas
        let num_posicoes = posicoes.len();
        let total_combinacoes = 9_usize.pow(num_posicoes as u32);
        
        for i in 0..total_combinacoes {
            let mut candidato = digitos_cpf.to_vec();
            let mut temporario = i;
            
            // Converter o índice em uma combinação de dígitos
            for &posicao in posicoes.iter() {
                let digito_original = digitos_cpf[posicao].to_digit(10).unwrap() as usize;
                let deslocamento = temporario % 9;
                temporario /= 9;
                
                // Calcular o novo dígito (pulando o dígito original)
                let novo_digito = if deslocamento < digito_original {
                    deslocamento
                } else {
                    deslocamento + 1
                };
                
                candidato[posicao] = char::from_digit(novo_digito as u32, 10).unwrap();
            }
            
            self.total_verificados += 1;
            let candidato_str: String = candidato.iter().collect();
            
            if cpf::valid(&candidato_str) {
                let formatado = formatar_cpf(&candidato_str);
                
                // Verificar se já existe este resultado
                let ja_existe = self.resultados.iter().any(|r| r.cpf_formatado == formatado);
                
                if !ja_existe {
                    let num_diferencas = contar_diferencas(&self.cpf_original, &candidato_str);
                    self.resultados.push(Resultado {
                        cpf_formatado: formatado,
                        num_diferencas,
                    });
                    *encontrado = true;
                }
            }
        }
    }

    fn resetar(&mut self) {
        let logo_texture = self.logo_texture.clone();
        *self = Self::novo();
        self.logo_texture = logo_texture; // Preservar a textura carregada
    }
}

impl eframe::App for Aplicativo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Continuar atualizando durante a busca
        if self.estado == EstadoApp::Buscando {
            ctx.request_repaint();
        }

        // Carregar logo se ainda não foi carregada
        self.carregar_logo(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            
            // Logo
            ui.vertical_centered(|ui| {
                if let Some(texture) = &self.logo_texture {
                    let largura_original = texture.size()[0] as f32;
                    let altura_original = texture.size()[1] as f32;
                    let escala = 0.6; // 60% do tamanho original
                    let tamanho_desejado = egui::vec2(
                        largura_original * escala,
                        altura_original * escala
                    );
                    ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(
                        texture.id(),
                        tamanho_desejado,
                    )));
                }
                
                ui.add_space(5.0);
                ui.label(
                    egui::RichText::new("Software de cunho técnico-cultural sobre a arte milenar do \"foi sem querer\"")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(150, 150, 150))
                        .italics()
                );
            });
            
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Área de entrada
            ui.horizontal(|ui| {
                ui.label("Digite o CPF:");
                
                let entrada_habilitada = matches!(self.estado, EstadoApp::Entrada | EstadoApp::Concluido);
                ui.add_enabled(
                    entrada_habilitada,
                    egui::TextEdit::singleline(&mut self.entrada)
                        .desired_width(200.0)
                );
                
                if ui.add_enabled(
                    entrada_habilitada && !self.entrada.is_empty(),
                    egui::Button::new("🔍 Buscar")
                ).clicked() {
                    self.validar_e_buscar();
                }

                if self.estado == EstadoApp::Concluido {
                    if ui.button("🔄 Nova Busca").clicked() {
                        self.resetar();
                    }
                }
            });

            ui.add_space(10.0);

            // Mensagem de erro
            if let Some(erro) = &self.mensagem_erro {
                ui.colored_label(egui::Color32::from_rgb(255, 100, 100), erro);
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Barra de status
            match self.estado {
                EstadoApp::Entrada => {
                    ui.vertical(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(150, 150, 150),
                            "Aguardando entrada..."
                        );
                        ui.label(
                            egui::RichText::new("💡 Gerando variações mínimas com rigor matemático e um pouquinho de falcatrua")
                                .size(10.0)
                                .color(egui::Color32::from_rgb(120, 120, 120))
                                .italics()
                        );
                    });
                }
                EstadoApp::Validando => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Validando CPF... (com todo respeito ao checksum)");
                    });
                }
                EstadoApp::Buscando => {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(format!(
                                "{} (Verificados: {})",
                                self.progresso_busca, self.total_verificados
                            ));
                        });
                        ui.label(
                            egui::RichText::new("🔎 Aplicando desatenção assistida por computador...")
                                .size(10.0)
                                .color(egui::Color32::from_rgb(120, 120, 120))
                                .italics()
                        );
                    });
                }
                EstadoApp::Concluido => {
                    ui.vertical(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(100, 255, 100),
                            &self.progresso_busca
                        );
                        if !self.resultados.is_empty() {
                            ui.label(
                                egui::RichText::new("✅ CPFs prontos para uso administrativamente ambíguo")
                                    .size(10.0)
                                    .color(egui::Color32::from_rgb(120, 200, 120))
                                    .italics()
                            );
                        } else {
                            ui.label(
                                egui::RichText::new("🤷 Nenhuma variação encontrada. Tente outro CPF.")
                                    .size(10.0)
                                    .color(egui::Color32::from_rgb(200, 200, 120))
                                    .italics()
                            );
                        }
                    });
                }
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Área de resultados
            ui.horizontal(|ui| {
                ui.heading(format!("Variações Válidas ({})", self.resultados.len()));
                if !self.resultados.is_empty() {
                    ui.label(
                        egui::RichText::new("— Alto potencial de confusão honesta")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(120, 120, 120))
                            .italics()
                    );
                }
            });
            ui.add_space(5.0);

            if !self.resultados.is_empty() {
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for resultado in &self.resultados {
                            ui.horizontal(|ui| {
                                ui.colored_label(
                                    egui::Color32::from_rgb(100, 255, 100),
                                    "✓"
                                );
                                ui.label(
                                    egui::RichText::new(&resultado.cpf_formatado)
                                        .monospace()
                                        .color(egui::Color32::from_rgb(150, 255, 150))
                                );
                                ui.colored_label(
                                    egui::Color32::from_rgb(150, 150, 150),
                                    format!(
                                        "({} dígito{} diferente{})",
                                        resultado.num_diferencas,
                                        if resultado.num_diferencas == 1 { "" } else { "s" },
                                        if resultado.num_diferencas == 1 { "" } else { "s" }
                                    )
                                );
                            });
                        }
                    });
            } else if self.estado != EstadoApp::Entrada {
                ui.colored_label(
                    egui::Color32::from_rgb(150, 150, 150),
                    "Nenhum resultado ainda..."
                );
            }

            // Rodapé com aviso legal satírico
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("⚠️ Este é um experimento educacional, satírico e culturalmente relevante")
                        .size(9.0)
                        .color(egui::Color32::from_rgb(100, 100, 100))
                );
                ui.label(
                    egui::RichText::new("Se for pego, negue com educação. Funciona desde 1500.")
                        .size(9.0)
                        .color(egui::Color32::from_rgb(100, 100, 100))
                        .italics()
                );
            });
        });
    }
}

fn gerar_combinacoes(posicoes: &[usize], num_mudancas: usize) -> Vec<Vec<usize>> {
    let mut resultado = Vec::new();
    let mut combinacao = Vec::new();
    gerar_combinacoes_auxiliar(posicoes, num_mudancas, 0, &mut combinacao, &mut resultado);
    resultado
}

fn gerar_combinacoes_auxiliar(
    posicoes: &[usize],
    num_mudancas: usize,
    inicio: usize,
    combinacao: &mut Vec<usize>,
    resultado: &mut Vec<Vec<usize>>,
) {
    if combinacao.len() == num_mudancas {
        resultado.push(combinacao.clone());
        return;
    }

    for i in inicio..posicoes.len() {
        combinacao.push(posicoes[i]);
        gerar_combinacoes_auxiliar(posicoes, num_mudancas, i + 1, combinacao, resultado);
        combinacao.pop();
    }
}

fn contar_diferencas(cpf1: &str, cpf2: &str) -> usize {
    cpf1.chars()
        .zip(cpf2.chars())
        .filter(|(c1, c2)| c1 != c2)
        .count()
}

fn formatar_cpf(cpf: &str) -> String {
    format!(
        "{}.{}.{}-{}",
        &cpf[0..3],
        &cpf[3..6],
        &cpf[6..9],
        &cpf[9..11]
    )
}

fn main() -> eframe::Result<()> {
    let opcoes = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 700.0])
            .with_min_inner_size([500.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "CPF QUILINGUE",
        opcoes,
        Box::new(|_cc| Ok(Box::new(Aplicativo::novo()))),
    )
}

use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum InstallmentCmd {
    /// Adiciona um novo parcelamento
    Add(AddInstallmentArgs),

    /// Lista os parcelamentos
    List {
        #[arg(long, short)]
        limit: Option<usize>,
    },

    /// Mostra detalhes de um parcelamento e suas faturas
    Show {
        #[arg(short, long)]
        id: i32,
    },

    /// Ajusta os centavos de uma parcela específica
    Adjust(AdjustInstallmentArgs),

    /// Deleta um parcelamento (reverte as faturas associadas se possível)
    Delete {
        #[arg(short, long)]
        id: i32,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum InstallmentError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Parse error: {0}")]
    Parse(String),
}

impl TryFrom<AddInstallmentArgs> for crate::models::installment::NewInstallment {
    type Error = InstallmentError;

    fn try_from(args: AddInstallmentArgs) -> Result<Self, Self::Error> {
        use std::str::FromStr;
        Ok(Self {
            credit_card_id: args.card_id,
            category_id: args.category_id,
            description: crate::models::types::NonEmptyString::from_str(&args.description)
                .map_err(|e| InstallmentError::Parse(e.to_string()))?,
            total_amount: crate::models::types::PositiveAmount::from_str(&args.total_amount)
                .map_err(|e| InstallmentError::Parse(e.to_string()))?,
            installments_count: args.installments_count,
            date: match args.date {
                Some(d) => chrono::NaiveDate::from_str(&d)
                    .map_err(|e| InstallmentError::Parse(e.to_string()))?,
                None => chrono::Utc::now().naive_local().date(),
            },
        })
    }
}

impl InstallmentCmd {
    pub async fn handle(self, ctx: &crate::context::AppContext) -> Result<(), InstallmentError> {
        match self {
            Self::Add(args) => crate::handlers::installment::add(ctx, args).await,
            Self::List { limit } => crate::handlers::installment::list(ctx, limit).await,
            Self::Show { id } => crate::handlers::installment::show(ctx, id).await,
            Self::Adjust(args) => crate::handlers::installment::adjust(ctx, args).await,
            Self::Delete { id } => crate::handlers::installment::delete(ctx, id).await,
        }
    }
}

#[derive(Debug, Args)]
pub struct AddInstallmentArgs {
    /// ID do cartão de crédito
    #[arg(long)]
    pub card_id: i32,

    /// ID da categoria (opcional)
    #[arg(long)]
    pub category_id: Option<i32>,

    /// Descrição da compra
    #[arg(short, long)]
    pub description: String,

    /// Valor total da compra parcelada
    #[arg(short, long)]
    pub total_amount: String,

    /// Quantidade de parcelas
    #[arg(short, long)]
    pub installments_count: i16,

    /// Data da compra (formato YYYY-MM-DD). Padrão: hoje
    #[arg(long)]
    pub date: Option<String>,
}

#[derive(Debug, Args)]
pub struct AdjustInstallmentArgs {
    /// ID do parcelamento base
    pub id: i32,

    /// Número da parcela a ser ajustada (ex: 1, 2, 3)
    #[arg(short, long)]
    pub number: i16,

    /// Novo valor exato da parcela (ex: 33.34)
    #[arg(short, long)]
    pub amount: String,
}

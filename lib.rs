#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod reporte {
    use sistema::SistemaRef;

    
    #[ink(storage)]
    pub struct Reporte {
        
        sistema: SistemaRef,
    }

    impl Reporte {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(sistema: SistemaRef) -> Self {
            Self { sistema }
        }
        
        #[ink(message)]
        pub fn ver_resultados(&self, id_de_votacion:i32){
            if let Some(v) = self.sistema.get_votacion(id_de_votacion){
                
            }else{
                panic!("NO EXISTE VOTACION DE ID: {}",id_de_votacion);
            }
        }

    }

}
